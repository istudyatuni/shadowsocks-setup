use std::{fs, path::PathBuf, str::FromStr};

use anyhow::{anyhow, bail, Context, Result};
use pnet::datalink;
use serde_json::{json, to_string_pretty};
use xshell::{cmd, Shell};

use super::input::shadowsocks::Install;
use crate::{args::InstallArgs, github::get_latest_release_tag, version::Version};

const DL_URL: &str = "https://github.com/shadowsocks/shadowsocks-rust/releases/download";

const SSSERVICE_BIN: &str = "/usr/local/bin/ssservice";
const CONFIG_FILE: &str = "/etc/sssconfig.json";

const SYSTEMD_SERVICE_FOLDER: &str = "/lib/systemd/system";
const SYSTEMD_SERVICE_FILE: &str = "/lib/systemd/system/ssserver.service";
const SYSTEMD_SERVICE_TEXT: &str = include_str!("../../static/ssserver.service");

const JOURNALD_CONF_FOLDER: &str = "/usr/lib/systemd/journald.conf.d";
const JOURNALD_CONF: &str = "/usr/lib/systemd/journald.conf.d/90-ssserver-tweaks.conf";
const JOURNALD_CONF_DATA: &str = include_str!("../../static/journald.conf");

const SYSCTL_CONF: &str = "/etc/sysctl.d/90-ssserver-tweaks.conf";
const SYSCTL_CONF_DATA: &str = include_str!("../../static/sysctl.conf");

pub fn install(sh: &Shell, args: InstallArgs) -> Result<()> {
    let installed_version = get_installed_version(sh);
    let latest_version = if let Some(version) = &args.version {
        version.clone()
    } else {
        eprintln!("[install] loading latest version");
        get_latest_ss_version()?
    };
    let install = Install::ask(args, installed_version, latest_version)?;

    check_requirements(sh)?;
    download(sh, &install)?;
    configure(sh, &install)?;
    print_config(sh, &install)?;

    cmd!(sh, "reboot").run().context("failed to reboot")?;

    Ok(())
}

pub fn undo(sh: &Shell) -> Result<()> {
    cmd!(sh, "systemctl disable ssserver").run()?;

    let to_backup = [CONFIG_FILE];
    for f in to_backup {
        let mut new_name = format!("{f}.bak");
        // if backup already exists, find first non-existing name like "{f}.bak1"
        if PathBuf::from(&new_name).exists() {
            if let Some(name) = (1..)
                .map(|i| format!("{new_name}{i}"))
                .find(|name| PathBuf::from(name).exists())
            {
                new_name = name;
            }
        }
        match fs::rename(f, &new_name) {
            Ok(_) => println!("[undo] saved {f} to {new_name}"),
            Err(e) => eprintln!("Couldn't remove {f}: {e}"),
        };
    }

    let to_remove = [
        SSSERVICE_BIN,
        SYSTEMD_SERVICE_FILE,
        SYSCTL_CONF,
        JOURNALD_CONF,
    ];
    for f in to_remove {
        match fs::remove_file(f) {
            Ok(_) => println!("[undo] removed {f}"),
            Err(e) => eprintln!("Couldn't remove {f}: {e}"),
        };
    }

    Ok(())
}

fn get_latest_ss_version() -> Result<Version> {
    get_latest_release_tag("shadowsocks", "shadowsocks-rust")
        .context("failed to get latest release")?
        .parse()
        .map_err(|e| anyhow!("{e}"))
        .context("got invalid version from latest release")
}

fn get_installed_version(sh: &Shell) -> Option<Version> {
    let exe = PathBuf::from(SSSERVICE_BIN);

    if !exe.exists() {
        return None;
    }

    let output = cmd!(sh, "{exe} -V").output().ok()?.stdout;
    let version = std::str::from_utf8(&output)
        .ok()?
        .split_whitespace()
        .last()?;
    Version::from_str(version).ok()
}

fn check_requirements(sh: &Shell) -> Result<()> {
    println!("[prepare] checking required executables");
    let bin_reqs = vec![
        "wget",
        "sha256sum",
        "tar",
        "systemctl",
        "cp",
        "sysctl",
        "ufw",
    ];
    let mut missed = false;
    for r in bin_reqs {
        if cmd!(sh, "which {r}").quiet().ignore_stdout().run().is_err() {
            missed = true;
            eprintln!("[error] {r} not found");
        }
    }

    if missed {
        bail!("some required executables is not found")
    }

    Ok(())
}

fn download(sh: &Shell, install: &Install) -> Result<()> {
    let version = &install.version;

    let url = download_url(version);
    cmd!(sh, "wget --no-clobber {url}").run()?;
    cmd!(sh, "wget --no-clobber {url}.sha256").run()?;

    let file = archive_filename(version);
    cmd!(sh, "sha256sum --check {file}.sha256").run()?;

    let version: &str = version;
    fs::create_dir_all(version).context("failed to create version dir for artifacts")?;
    cmd!(sh, "tar -xf {file} -C {version}").run()?;
    cmd!(sh, "cp ssservice {SSSERVICE_BIN}").run()?;

    Ok(())
}

fn configure(sh: &Shell, install: &Install) -> Result<()> {
    println!("\n[config] create shadowsocks config");
    let sssconfig = json!({
        "server": "0.0.0.0",
        "server_port": install.server_port,
        "password": install.server_password,
        "method": install.cipher.to_string(),
    });
    fs::write(CONFIG_FILE, to_string_pretty(&sssconfig)?)?;

    println!("\n[config] create shadowsocks systemd service unit");
    fs::create_dir_all(SYSTEMD_SERVICE_FOLDER)?;
    fs::write(SYSTEMD_SERVICE_FILE, SYSTEMD_SERVICE_TEXT)?;

    cmd!(sh, "systemctl enable ssserver").run()?;
    cmd!(sh, "systemctl restart ssserver").run()?;

    let journald_conf = PathBuf::from(JOURNALD_CONF);
    if !journald_conf.exists() {
        fs::create_dir_all(JOURNALD_CONF_FOLDER)?;
        println!("\n[config] setting new log storing policy in {JOURNALD_CONF}");
        fs::write(journald_conf, JOURNALD_CONF_DATA)?;
    }

    let sysctl_conf = PathBuf::from(SYSCTL_CONF);
    if !sysctl_conf.exists() {
        println!("\n[config] setting kernel tweaks in {SYSCTL_CONF}");
        fs::write(sysctl_conf, SYSCTL_CONF_DATA)?;
        // apply
        cmd!(sh, "sysctl -p").run()?;
    }

    println!("\n[config] opening firewall ports");
    cmd!(sh, "ufw allow 22").run()?;
    let port = install.server_port.to_string();
    cmd!(sh, "ufw allow {port}").run()?;
    cmd!(sh, "ufw --force enable").run()?;

    Ok(())
}

fn print_config(sh: &Shell, install: &Install) -> Result<()> {
    let all_interfaces = datalink::interfaces();
    let default_interface = all_interfaces
        .iter()
        .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty())
        .expect("Couldn't find IP address");
    let server_ip = default_interface
        .ips
        .iter()
        .find(|e| e.is_ipv4())
        .expect("No IPv4 address")
        .ip();

    let client_config = json!({
        "server": server_ip,
        "server_port": install.server_port,
        "local_port": 1080,
        "password": install.server_password,
        "method": install.cipher.to_string(),
    });
    let client_config_path = "sssconfig-client.json";
    let client_config = to_string_pretty(&client_config)?;
    std::fs::write(client_config_path, &client_config).context("failed to write client config")?;
    let share_url = cmd!(sh, "./ssurl -e {client_config_path}").quiet().read()?;

    println!("####### CLIENT CONFIG #######");
    println!("{client_config}");
    println!("#############################");
    println!("Share URL: {share_url}");
    println!("#############################");

    Ok(())
}

fn archive_filename(version: &Version) -> String {
    format!(
        "shadowsocks-{}.x86_64-unknown-linux-gnu.tar.xz",
        version.as_prefixed()
    )
}

fn download_url(version: &Version) -> String {
    DL_URL.to_owned()
        + "/"
        + version.as_prefixed().as_str()
        + "/"
        + archive_filename(version).as_str()
}
