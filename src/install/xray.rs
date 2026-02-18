use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use xshell::{Shell, cmd};

use crate::{
    args::{XrayInstallArgs, XrayInstallStep},
    github::get_latest_release_tag,
    install::{
        check_requirements, create_and_cd_to_artifacts_dir, network::open_firewall_ports_and_enable,
    },
    version::Version,
};

const DL_URL: &str = "https://github.com/XTLS/Xray-core/releases/download";
const DL_FILE: &str = "Xray-linux-64.zip";

const CRON_DIR: &str = "/etc/cron.d";
const SYSTEMD_DIR: &str = "/etc/systemd/system";
const NGINX_DIR: &str = "/etc/nginx";
const XRAY_ETC_DIR: &str = "/usr/local/etc/xray";
const XRAY_BIN: &str = "/usr/local/bin/xray";

const VLESS_INBOUND_TAG: &str = "vless";

const ACME_RENEW_SH: &str = include_str!("../../static/acme-renew.sh");
const NGINX_CONF: &str = include_str!("../../static/nginx.conf");
const XRAY_SERVICE: &str = include_str!("../../static/xray.service");
const XRAY_API_CONF: &str = include_str!("../../static/xray_01_api.json");
const XRAY_BASE_CONF: &str = include_str!("../../static/xray_03_base.json");

const CRON_RENEW_CERT: &str = include_str!("../../static/cert-renew.cron");
const CRON_RENEW_DOMAIN: &str = include_str!("../../static/domain-renew.cron");

const INSTALL_EXE_REQUIRED: &[&str] = &[
    "chmod",
    "nginx",
    "sh",
    "sha512sum",
    "systemctl",
    "ufw",
    "unzip",
    "wget",
];

const STATE_FILE: &str = "/tmp/xray-install-state.json";

pub fn run_install_manager(sh: &Shell, args: XrayInstallArgs) -> Result<()> {
    create_and_cd_to_artifacts_dir(sh)?;

    let home = std::env::var("HOME")
        .inspect_err(|e| eprintln!("failed to get HOME variable, using /root: {e}"))
        .unwrap_or_else(|_| "/root".to_string());

    let state = InstallState {
        args,
        home_dir: PathBuf::from(&home),
        home_dir_str: home,
        download_dir: None,
        cert_dir: None,
    };
    let state =
        serde_json::to_string_pretty(&state).context("failed to serialize install state")?;
    std::fs::write(STATE_FILE, state).context("failed to save install state")?;

    let self_bin = std::env::current_exe().context("failed to get current exe")?;
    for step in XrayInstallStep::values() {
        let step = step.to_string();
        cmd!(sh, "{self_bin} xray install-step {step}").run()?;
    }

    Ok(())
}

pub fn install(sh: &Shell, step: XrayInstallStep) -> Result<()> {
    let state = std::fs::read_to_string(STATE_FILE).context("failed to read install state")?;
    let mut state: InstallState =
        serde_json::from_str(&state).context("failed to deserialize install state")?;
    let args = &state.args;

    let mut should_save_state = false;
    match step {
        XrayInstallStep::DownloadXray => {
            let latest_version = get_latest_xray_version()?;
            eprintln!("[install] latest version: {}", latest_version.as_prefixed());

            check_requirements(sh, INSTALL_EXE_REQUIRED)?;
            let dl_dir = sh.current_dir().join(latest_version.to_string());
            download(sh, &latest_version, &dl_dir)?;
            state.download_dir = Some(dl_dir);
            should_save_state = true;
        }
        XrayInstallStep::InstallXray => {
            let Some(dl_dir) = &state.download_dir else {
                bail!("invalid state: no download_dir")
            };
            install_xray(sh, dl_dir)?;
        }
        XrayInstallStep::ConfigureFirewall => {
            open_firewall_ports_and_enable(sh, &[22, 80, 443])?;
        }
        XrayInstallStep::ConfigureCert => {
            let acme = configure_cert(sh, args, &state.home_dir)?;
            state.cert_dir = Some(acme.cert_dir);
            should_save_state = true;
        }
        XrayInstallStep::ConfigureElse => {
            let Some(cert_dir) = &state.cert_dir else {
                bail!("invalid state: no download_dir")
            };
            let mut users_config = XrayConfig::new(cert_dir)?;
            configure(args, &mut users_config, cert_dir, &state.home_dir_str)?;
            start_services(sh)?;
            print_users_links(users_config.users(), &args.domain);
        }
    }

    if should_save_state {
        let state =
            serde_json::to_string_pretty(&state).context("failed to serialize install state")?;
        std::fs::write(STATE_FILE, state).context("failed to save install state")?;
    }

    Ok(())
}

fn get_latest_xray_version() -> Result<Version> {
    get_latest_release_tag("XTLS", "Xray-core")
        .context("failed to get latest release")?
        .parse()
        .map_err(|e| anyhow!("{e}"))
        .context("got invalid version from latest release")
}

fn download(sh: &Shell, version: &Version, dl_dir: &Path) -> Result<()> {
    create_dir(dl_dir)?;
    let url = download_url(version);

    let _new_dir = sh.push_dir(dl_dir);

    cmd!(sh, "wget --no-clobber {url}").run()?;
    cmd!(sh, "wget --no-clobber {url}.dgst").run()?;

    let file = DL_FILE;
    let hash = cmd!(sh, "sha512sum {file}")
        .read()
        .context("failed to read sha512sum output")?;
    let Some(hash) = hash.split_whitespace().next() else {
        bail!("hash not found in sha512sum output")
    };

    let dgst = std::fs::read_to_string(sh.current_dir().join(format!("{file}.dgst")))
        .context("failed to read .dgst file")?;
    if !dgst.contains(hash) {
        eprintln!(".dgst file:\n{dgst}");
        bail!("hash check failed, expected sha512 hash not found, hash: {hash}")
    }

    cmd!(sh, "unzip -u {file}").run()?;

    drop(_new_dir);

    Ok(())
}

fn install_xray(sh: &Shell, dl_dir: &Path) -> Result<()> {
    let _new_dir = sh.push_dir(dl_dir);

    std::fs::rename(sh.current_dir().join("xray"), XRAY_BIN)
        .context("failed to move xray to bin dir")?;

    let dir = "/usr/local/share/xray";
    eprintln!("creating directory {dir}");
    std::fs::create_dir_all(dir).with_context(|| format!("failed to create {dir}"))?;
    for file in ["geoip.dat", "geosite.dat"] {
        let source = sh.current_dir().join(file);
        eprintln!("moving {} to {dir}", source.display());
        std::fs::rename(source, format!("{dir}/{file}"))
            .with_context(|| format!("failed to move {file} to {dir}"))?;
    }

    drop(_new_dir);

    Ok(())
}

#[cfg_attr(feature = "fake-cert", expect(unused))]
fn configure_cert(
    sh: &Shell,
    args: &XrayInstallArgs,
    home_dir: &Path,
) -> Result<AcmeInstallResult> {
    let cert_dir = home_dir.join("xray-cert");
    create_dir(&cert_dir)?;

    #[cfg(feature = "fake-cert")]
    {
        eprintln!("[install cert] creating fake cert");
        std::fs::write(
            cert_dir.join("xray.crt"),
            include_str!("../../static/test/fake.crt"),
        )
        .context("failed to create xray.crt")?;
        std::fs::write(
            cert_dir.join("xray.key"),
            include_str!("../../static/test/fake.key"),
        )
        .context("failed to create xray.key")?;
        return Ok(AcmeInstallResult { cert_dir });
    }

    let domain = &args.domain;
    let acme_bin = home_dir.join(".acme.sh/acme.sh");
    const ACME_INSTALLER: &str = "/tmp/acme-install.sh";
    if !PathBuf::from(ACME_INSTALLER).exists() {
        cmd!(
            sh,
            "wget --no-clobber -O {ACME_INSTALLER} https://get.acme.sh"
        )
        .run()?;
    }
    if !acme_bin.exists() {
        cmd!(sh, "sh {ACME_INSTALLER}").run()?;
    }

    cmd!(sh, "{acme_bin} --upgrade --auto-upgrade").run()?;

    cmd!(sh, "{acme_bin} --set-default-ca --server zerossl").run()?;
    let email = &args.zerossl_email;
    cmd!(
        sh,
        "{acme_bin} --register-account -m {email} --debug 2 --output-insecure"
    )
    .run()?;
    cmd!(
        sh,
        "{acme_bin} --issue -d {domain} --keylength ec-256 --nginx"
    )
    .run()?;

    cmd!(sh, "{acme_bin} --install-cert -d {domain} --ecc --fullchain-file {cert_dir}/xray.crt --key-file {cert_dir}/xray.key").run()?;
    cmd!(sh, "chmod +r {cert_dir}/xray.key").run()?;

    Ok(AcmeInstallResult { cert_dir })
}

fn configure(
    args: &XrayInstallArgs,
    users_config: &mut XrayConfig,
    cert_dir: &Path,
    home: &str,
) -> Result<()> {
    let cron_dir = PathBuf::from(CRON_DIR);

    let domain = &args.domain;
    let vars = [
        ("VAR_HOME", home.to_string()),
        ("VAR_DOMAIN", domain.clone()),
        (
            "VAR_DOMAIN_RENEW_URL",
            args.domain_renew_url
                .clone()
                .unwrap_or_else(|| "NOT_SET".to_string()),
        ),
        ("VAR_VLESS_INBOUND_TAG", VLESS_INBOUND_TAG.to_string()),
        ("VAR_XRAY_BIN", XRAY_BIN.to_string()),
        ("VAR_XRAY_API_PORT", args.api_port.to_string()),
        ("VAR_XRAY_ETC_DIR", XRAY_ETC_DIR.to_string()),
    ];
    let replace_vars = |text: &str| {
        let mut res = text.to_string();
        for (name, value) in &vars {
            res = res.replace(name, value);
        }
        // todo: check no VAR_ remains
        res
    };

    let save_config = |dir: &Path, file: &str, text: &str| -> Result<()> {
        let data = replace_vars(text);
        let path = dir.join(file);
        eprintln!("writing {}", path.display());
        std::fs::write(path, data)
            .with_context(|| format!("failed to save {file} to {}", dir.display()))?;
        Ok(())
    };

    // xray configs

    let etc = PathBuf::from(XRAY_ETC_DIR);
    create_dir(&etc)?;
    if args.api {
        // writing 01_api before 05_main because inbound[0] from 01_api should
        // be before other rules in 05_main after loading
        save_config(&etc, "01_api.json", XRAY_API_CONF)?;
    }
    save_config(&etc, "03_base.json", XRAY_BASE_CONF)?;
    if !args.add_user_ids.is_empty() {
        users_config.reserve_users_space(args.add_user_ids.len());
        for id in &args.add_user_ids {
            users_config.add_user_with_id(id);
        }
    } else {
        users_config.add_users(args.add_users_count);
    }
    let config_data =
        serde_json::to_string_pretty(&users_config).context("failed to serialize xray config")?;
    save_config(&etc, "05_main.json", &config_data)?;
    drop(etc);

    // systemd config

    let systemd = PathBuf::from(SYSTEMD_DIR);
    create_dir(&systemd)?;
    save_config(&systemd, "xray.service", XRAY_SERVICE)?;

    // nginx config

    let nginx = PathBuf::from(NGINX_DIR);
    create_dir(&nginx)?;
    save_config(&nginx, "nginx.conf", NGINX_CONF)?;

    // cron config

    if args.domain_renew_url.is_some() {
        save_config(&cron_dir, "domain-renew", CRON_RENEW_DOMAIN)?;
    }

    // acme cron

    save_config(cert_dir, "renew.sh", ACME_RENEW_SH)?;
    save_config(&cron_dir, "cert-renew", CRON_RENEW_CERT)?;

    Ok(())
}

fn start_services(sh: &Shell) -> Result<()> {
    cmd!(sh, "systemctl enable --now xray").run()?;
    cmd!(sh, "systemctl enable --now nginx").run()?;
    Ok(())
}

fn print_users_links(users: &[Client], domain: &str) {
    eprintln!("users links:");
    const NAME: &str = "xray";
    for u in users {
        println!(
            "vless://{}@{domain}:443/?type=tcp&encryption=none&flow=xtls-rprx-vision&security=tls&fp=chrome#{NAME}",
            u.id
        );
    }
}

fn download_url(version: &Version) -> String {
    DL_URL.to_owned() + "/" + version.as_prefixed().as_str() + "/" + DL_FILE
}

fn create_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        eprintln!("creating directory {}", path.display());
        std::fs::create_dir_all(path)
            .with_context(|| format!("failed to create {}", path.display()))?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct InstallState {
    args: XrayInstallArgs,
    home_dir: PathBuf,
    home_dir_str: String,
    download_dir: Option<PathBuf>,
    cert_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct XrayConfig {
    routing: RoutingConfig,
    inbounds: Vec<InboundConfig>,

    #[serde(skip_serializing)]
    inbound_with_clients_index: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RoutingConfig {
    domain_strategy: String,
    rules: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct InboundConfig {
    tag: String,
    settings: InboundConfigSettings,

    #[serde(flatten)]
    rest: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct InboundConfigSettings {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    clients: Vec<Client>,

    #[serde(flatten)]
    rest: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct Client {
    id: String,
    flow: String,
}

impl XrayConfig {
    fn new(cert_dir: &Path) -> Result<Self> {
        let routing_rules = vec![
            // xray ignores this rule if api is not specified
            json!({
              "inboundTag": [
                "api"
              ],
              "outboundTag": "api"
            }),
            json!({
              "ip": [
                "geoip:private"
              ],
              "outboundTag": "block"
            }),
            json!({
              "ip": [
                "geoip:cn"
              ],
              "outboundTag": "block"
            }),
            json!({
              "domain": [
                "geosite:category-ads-all"
              ],
              "outboundTag": "block"
            }),
        ];
        let vless_inbound_rule = InboundConfig {
            tag: "vless".to_string(),
            settings: InboundConfigSettings {
                clients: vec![],
                rest: json!({
                    "decryption": "none",
                    "fallbacks": [{ "dest": 8080 }]
                }),
            },
            rest: json!({
                "port": 443,
                "protocol": "vless",
                "streamSettings": {
                    "network": "tcp",
                    "security": "tls",
                    "tlsSettings": {
                        "alpn": "http/1.1",
                        "certificates": [
                            {
                                "certificateFile": path_to_str(&cert_dir.join("xray.crt"))?,
                                "keyFile": path_to_str(&cert_dir.join("xray.key"))?,
                            }
                        ]
                    }
                }
            }),
        };

        Ok(Self {
            routing: RoutingConfig {
                domain_strategy: "IPIfNonMatch".to_string(),
                rules: routing_rules,
            },
            inbounds: vec![vless_inbound_rule],
            inbound_with_clients_index: 0,
        })
    }
    fn users(&self) -> &[Client] {
        &self.inbounds[self.inbound_with_clients_index]
            .settings
            .clients
    }
    fn users_mut(&mut self) -> &mut Vec<Client> {
        &mut self.inbounds[self.inbound_with_clients_index]
            .settings
            .clients
    }
    fn reserve_users_space(&mut self, count: usize) {
        self.users_mut().reserve(count);
    }
    fn add_users(&mut self, count: usize) -> &mut Self {
        self.reserve_users_space(count);
        for _ in 0..count {
            self.add_user();
        }
        self
    }
    fn add_user(&mut self) -> &mut Self {
        self.add_user_with_id(Uuid::new_v4().to_string().as_str())
    }
    fn add_user_with_id(&mut self, id: &str) -> &mut Self {
        self.users_mut().push(Client {
            id: id.to_string(),
            flow: "xtls-rprx-vision".to_string(),
        });
        self
    }
}

struct AcmeInstallResult {
    cert_dir: PathBuf,
}

fn path_to_str(p: &Path) -> Result<String> {
    p.to_str()
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("path {} is not valid utf-8", p.display()))
}
