#![expect(unused)]

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};
use xshell::{Shell, cmd};

use crate::{
    args::XrayInstallArgs,
    github::get_latest_release_tag,
    install::{check_requirements, network::open_firewall_ports_and_enable},
    version::Version,
};

const DL_URL: &str = "https://github.com/XTLS/Xray-core/releases/download";
const DL_FILE: &str = "Xray-linux-64.zip";

const CRON_DIR: &str = "/etc/cron.d";
const SYSTEMD_DIR: &str = "/etc/systemd/system";
const NGINX_DIR: &str = "/etc/nginx";
const XRAY_ETC_DIR: &str = "/usr/local/etc/xray";
const XRAY_BIN: &str = "/usr/local/bin/xray";

const ACME_RENEW_SH: &str = include_str!("../../static/acme-renew.sh");
const NGINX_CONF: &str = include_str!("../../static/nginx.conf");
const XRAY_CONF: &str = include_str!("../../static/xray.json");
const XRAY_SERVICE: &str = include_str!("../../static/xray.service");

const CRON_RENEW_CERT: &str = include_str!("../../static/cert-renew.cron");
const CRON_RENEW_DOMAIN: &str = include_str!("../../static/domain-renew.cron");

const INSTALL_EXE_REQUIRED: &[&str] = &[
    "cp",
    "chmod",
    "sh",
    "sha512sum",
    // "sysctl",
    "systemctl",
    "ufw",
    "unzip",
    "wget",
];

pub fn install(sh: &Shell, args: XrayInstallArgs) -> Result<()> {
    let latest_version = get_latest_xray_version()?;
    eprintln!("[install] latest version: {}", latest_version.as_prefixed());

    check_requirements(sh, INSTALL_EXE_REQUIRED)?;
    download(sh, &latest_version)?;
    configure(sh, &args)?;

    open_firewall_ports_and_enable(sh, &[22, 443])?;

    unimplemented!()
}

fn get_latest_xray_version() -> Result<Version> {
    get_latest_release_tag("XTLS", "Xray-core")
        .context("failed to get latest release")?
        .parse()
        .map_err(|e| anyhow!("{e}"))
        .context("got invalid version from latest release")
}

fn download(sh: &Shell, version: &Version) -> Result<()> {
    let dl_dir = sh.current_dir().join(version.to_string());
    let url = download_url(version);
    if !dl_dir.exists() {
        std::fs::create_dir_all(&dl_dir).context("failed to create version dir for artifacts")?;
    }

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

    let dgst =
        std::fs::read_to_string(format!("{file}.dgst")).context("failed to read .dgst file")?;
    if !dgst.contains(hash) {
        eprintln!(".dgst file:\n{dgst}");
        bail!("hash check failed, expected sha512 hash not found, hash: {hash}")
    }

    cmd!(sh, "unzip {file}").run()?;
    std::fs::rename(sh.current_dir().join("xray"), XRAY_BIN)
        .context("failed to move xray to bin dir")?;

    let dir = "/usr/local/share/xray";
    std::fs::create_dir_all(dir).with_context(|| format!("failed to create {dir}"))?;
    for file in ["geoip.dat", "geosite.dat"] {
        std::fs::rename(
            sh.current_dir().join(file),
            format!("/usr/local/share/xray/{file}"),
        )
        .with_context(|| format!("failed to move {file} to {dir}"))?;
    }

    drop(_new_dir);

    Ok(())
}

fn configure(sh: &Shell, args: &XrayInstallArgs) -> Result<()> {
    let home = std::env::var("HOME")
        .inspect_err(|e| eprintln!("failed to get HOME variable, using /root"))
        .unwrap_or_else(|_| "/root".to_string());
    let cron_dir = PathBuf::from(CRON_DIR);

    let domain = &args.domain;
    let vars = [
        ("VAR_HOME", home.clone()),
        ("VAR_DOMAIN", domain.clone()),
        ("VAR_DOMAIN_RENEW_URL", "TODO".to_string()),
        ("VAR_XRAY_BIN", XRAY_BIN.to_string()),
        ("VAR_XRAY_ETC_DIR", XRAY_ETC_DIR.to_string()),
    ];
    let replace_vars = |text: &str| {
        let res = text.to_string();
        for (name, value) in &vars {
            res.replace(name, value);
        }
        // todo: check no VAR_ remains
        res
    };

    // configs

    let etc = PathBuf::from(XRAY_ETC_DIR);
    std::fs::create_dir_all(&etc).with_context(|| format!("failed to create {XRAY_ETC_DIR}"))?;

    let config_data = replace_vars(XRAY_CONF);
    std::fs::write(etc.join("05_main.json"), config_data)
        .with_context(|| format!("failed to save 05_main.json to {XRAY_ETC_DIR}"))?;

    let systemd = PathBuf::from(SYSTEMD_DIR);
    std::fs::create_dir_all(&systemd).with_context(|| format!("failed to create {SYSTEMD_DIR}"))?;
    let service_data = replace_vars(XRAY_SERVICE);
    std::fs::write(etc.join("xray.service"), service_data)
        .with_context(|| format!("failed to save xray.service to {SYSTEMD_DIR}"))?;

    let nginx = PathBuf::from(NGINX_DIR);
    if !nginx.exists() {
        std::fs::create_dir_all(&nginx).with_context(|| format!("failed to create {NGINX_DIR}"))?;
    }
    let nxing_data = replace_vars(NGINX_CONF);
    std::fs::write(etc.join("nginx.conf"), nxing_data)
        .with_context(|| format!("failed to save nginx.conf to {NGINX_DIR}"))?;

    if let Some(_url) = &args.domain_renew_url {
        let domain_renew_cron = replace_vars(CRON_RENEW_DOMAIN);
        let path = cron_dir.join("domain-renew");
        std::fs::write(path, domain_renew_cron).context("failed to write domain-renew cron")?;
    }

    // acme

    let home_path = PathBuf::from(home);
    let acme_bin = home_path.join(".acme.sh/acme.sh");
    const ACME_INSTALL: &str = "/tmp/acme-install.sh";
    if !PathBuf::from(ACME_INSTALL).exists() {
        cmd!(
            sh,
            "wget --no-clobber -O {ACME_INSTALL} https://get.acme.sh"
        )
        .run()?;
    }
    if !acme_bin.exists() {
        cmd!(sh, "sh {ACME_INSTALL}").run()?;
    }

    cmd!(sh, "{acme_bin} --upgrade --auto-upgrade").run()?;

    cmd!(sh, "{acme_bin} --set-default-ca --server zerossl").run()?;
    if let Some(email) = &args.zerossl_email {
	    cmd!(sh, "{acme_bin} --register-account -m {email}").run()?;
    }
    cmd!(
        sh,
        "{acme_bin} --issue -d {domain} --keylength ec-256 --nginx"
    )
    .run()?;

    let cert_dir = home_path.join("xray-cert");
    if !cert_dir.exists() {
        std::fs::create_dir_all(&cert_dir)
            .with_context(|| format!("failed to create {}", cert_dir.display()))?;
    }

    let cert_dir_str = cert_dir.display().to_string();
    cmd!(sh, "{acme_bin} --install-cert -d {domain} --ecc --fullchain-file {cert_dir_str}/xray.crt --key-file {cert_dir_str}/xray.key").run()?;
    cmd!(sh, "chmod +r {cert_dir_str}/xray.key").run()?;

    std::fs::write(cert_dir.join("renew.sh"), replace_vars(ACME_RENEW_SH))
        .context("failed to save renew.sh")?;

    let cert_renew_cron = replace_vars(CRON_RENEW_CERT);
    let path = cron_dir.join("cert-renew");
    std::fs::write(path, cert_renew_cron).context("failed to write cert-renew cron")?;

    todo!()
}

fn download_url(version: &Version) -> String {
    DL_URL.to_owned() + "/" + version.as_prefixed().as_str() + "/" + DL_FILE
}
