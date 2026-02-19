use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use xshell::{Shell, cmd};

use crate::{
    args::{XrayInstallArgs, XrayInstallStep},
    github::get_latest_release_tag,
    install::{
        check_requirements, create_and_cd_to_artifacts_dir,
        network::open_firewall_ports_and_enable, save_config,
    },
    version::Version,
};

use super::{
    create_dir,
    xray_config::{Client, XrayConfig},
};

const DL_URL: &str = "https://github.com/XTLS/Xray-core/releases/download";
const DL_FILE: &str = "Xray-linux-64.zip";

const CRON_DIR: &str = "/etc/cron.d";
const SYSTEMD_DIR: &str = "/etc/systemd/system";
const NGINX_DIR: &str = "/etc/nginx";
const XRAY_ETC_DIR: &str = "/usr/local/etc/xray";
const XRAY_BIN: &str = "/usr/local/bin/xray";

const VLESS_INBOUND_TAG: &str = "vless";

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

mod vars {
    macro_rules! vars {
        ($($var:ident),* $(,)?) => {
            $(
                pub const $var: &str = concat!("VAR_", stringify!($var));
            )*

            #[cfg(test)]
            pub const ALL_VARS: &[&str] = &[ $($var),* ];
        };
    }

    vars!(
        HOME,
        DOMAIN,
        DOMAIN_RENEW_URL,
        VLESS_INBOUND_TAG,
        XRAY_BIN,
        XRAY_API_PORT,
        XRAY_ETC_DIR,
    );
}

mod configs {
    macro_rules! configs {
        ($($name:ident = $path:literal),* $(,)?) => {
            $(
                pub const $name: &str = include_str!($path);
            )*

            #[cfg(test)]
            pub const ALL_CONFIGS: &[&str] = &[ $($name),* ];
        };
    }

    configs!(
        ACME_RENEW_SH = "../../static/acme-renew.sh",
        NGINX_CONF = "../../static/nginx.conf",
        XRAY_SERVICE = "../../static/xray.service",
        XRAY_API_CONF = "../../static/xray_01_api.json",
        XRAY_BASE_CONF = "../../static/xray_03_base.json",
        CRON_RENEW_CERT = "../../static/cert-renew.cron",
        CRON_RENEW_DOMAIN = "../../static/domain-renew.cron",
    );
}

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
        (vars::HOME, home.to_string()),
        (vars::DOMAIN, domain.clone()),
        (
            vars::DOMAIN_RENEW_URL,
            args.domain_renew_url
                .clone()
                .unwrap_or_else(|| "NOT_SET".to_string()),
        ),
        (vars::VLESS_INBOUND_TAG, VLESS_INBOUND_TAG.to_string()),
        (vars::XRAY_BIN, XRAY_BIN.to_string()),
        (vars::XRAY_API_PORT, args.api_port.to_string()),
        (vars::XRAY_ETC_DIR, XRAY_ETC_DIR.to_string()),
    ];
    let replace_vars = |text: &str| {
        let mut res = text.to_string();
        for (name, value) in &vars {
            res = res.replace(name, value);
        }
        res
    };

    let save_config = |dir: &Path, file: &str, text: &str| -> Result<()> {
        save_config(dir, file, &replace_vars(text))
    };

    // xray configs

    let etc = PathBuf::from(XRAY_ETC_DIR);
    create_dir(&etc)?;
    if args.api {
        // writing 01_api before 05_main because inbound[0] from 01_api should
        // be before other rules in 05_main after loading
        save_config(&etc, "01_api.json", configs::XRAY_API_CONF)?;
    }
    save_config(&etc, "03_base.json", configs::XRAY_BASE_CONF)?;
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
    save_config(&systemd, "xray.service", configs::XRAY_SERVICE)?;

    // nginx config

    let nginx = PathBuf::from(NGINX_DIR);
    create_dir(&nginx)?;
    save_config(&nginx, "nginx.conf", configs::NGINX_CONF)?;

    // cron config

    if args.domain_renew_url.is_some() {
        save_config(&cron_dir, "domain-renew", configs::CRON_RENEW_DOMAIN)?;
    }

    // acme cron

    save_config(cert_dir, "renew.sh", configs::ACME_RENEW_SH)?;
    save_config(&cron_dir, "cert-renew", configs::CRON_RENEW_CERT)?;

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

#[derive(Debug, Serialize, Deserialize)]
struct InstallState {
    args: XrayInstallArgs,
    home_dir: PathBuf,
    home_dir_str: String,
    download_dir: Option<PathBuf>,
    cert_dir: Option<PathBuf>,
}

struct AcmeInstallResult {
    cert_dir: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_vars_replaced() {
        fn replace_vars(text: &str) -> String {
            let mut res = text.to_string();
            for name in vars::ALL_VARS {
                res = res.replace(name, "");
            }
            res
        }
        let not_all_replaced = configs::ALL_CONFIGS
            .iter()
            .map(|s| replace_vars(s))
            .any(|s| s.contains("VAR_"));
        assert!(!not_all_replaced);
    }
}
