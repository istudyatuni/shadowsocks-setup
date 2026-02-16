#![expect(unused)]

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

const XRAY_BIN: &str = "/usr/local/bin/xray";

const ACME_RENEW_SH: &str = include_str!("../../static/acme-renew.sh");
const NGINX_CONF: &str = include_str!("../../static/nginx.conf");
const XRAY_CONF: &str = include_str!("../../static/xray.json");
const XRAY_SERVICE: &str = include_str!("../../static/xray.service");

const INSTALL_EXE_REQUIRED: &[&str] = &[
    "cp",
    "sha512sum",
    "sysctl",
    "systemctl",
    "ufw",
    "unzip",
    "wget",
];

pub fn install(sh: &Shell, args: XrayInstallArgs) -> Result<()> {
    let vars = [
        (
            "HOME",
            std::env::var("HOME")
                .inspect_err(|e| eprintln!("failed to get HOME variable"))
                .unwrap_or_else(|_| "/root".to_string()),
        ),
        ("DOMAIN", args.domain),
    ];
    let replace_vars = |text: &str| {
        let res = text.to_string();
        for (name, value) in &vars {
            res.replace(name, value);
        }
        res
    };

    let latest_version = get_latest_xray_version()?;
    eprintln!("[install] latest version: {}", latest_version.as_prefixed());

    check_requirements(sh, INSTALL_EXE_REQUIRED)?;
    download(sh, &latest_version)?;

    // open_firewall_ports_and_enable(sh, &[22, 443])?;

    unimplemented!()
}

fn get_latest_xray_version() -> Result<Version> {
    get_latest_release_tag("XTLS", "Xray-core")
        .context("failed to get latest release")?
        .parse()
        .map_err(|e| anyhow!("{e}"))
        .context("got invalid version from latest release")
}

#[expect(clippy::trim_split_whitespace)]
fn download(sh: &Shell, version: &Version) -> Result<()> {
    let url = download_url(version);
    std::fs::create_dir_all(version.to_string())
        .context("failed to create version dir for artifacts")?;

    let _new_dir = sh.push_dir(version.to_string());

    cmd!(sh, "wget --no-clobber {url}").run()?;
    cmd!(sh, "wget --no-clobber {url}.dgst").run()?;

    let file = DL_FILE;
    let hash = cmd!(sh, "sha256sum {file}")
        .read()
        .context("failed to read sha256sum output")?;
    let Some(hash) = hash.trim().split_whitespace().next() else {
        bail!("hash not found in sha256sum output")
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

    std::fs::create_dir_all("/usr/local/etc/xray")
        .context("failed to create /usr/local/etc/xray")?;

    drop(_new_dir);

    Ok(())
}

fn download_url(version: &Version) -> String {
    DL_URL.to_owned() + "/" + version.as_prefixed().as_str() + "/" + DL_FILE
}
