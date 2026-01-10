use std::fs;
use std::io::Write;

use anyhow::{bail, Context, Result};
use pnet::datalink;
use serde_json::{json, to_string_pretty};
use xshell::{cmd, Shell};

use crate::state::{Action, Install, State};

const DL_URL: &str = "https://github.com/shadowsocks/shadowsocks-rust/releases/download";

const SSSERVICE_BIN: &str = "/usr/local/bin/ssservice";
const CONFIG_FILE: &str = "/etc/sssconfig.json";

const SYSTEMD_SERVICE_FOLDER: &str = "/lib/systemd/system";
const SYSTEMD_SERVICE_FILE: &str = "/lib/systemd/system/ssserver.service";
const SYSTEMD_SERVICE_TEXT: &str = include_str!("../../static/ssserver.service");

const CONFIGS_CHECK_HEADER: &str = "# shadowsocks tweaks";

const JOURNALD_CONF: &str = "/etc/systemd/journald.conf";
const JOURNALD_CONF_TAIL: &str = include_str!("../../static/journald-tail.conf");

const SYSCTL_CONF: &str = "/etc/sysctl.conf";
const SYSCTL_CONF_TAIL: &str = include_str!("../../static/sysctl-tail.conf");

// common

fn archive_filename(version: &str) -> String {
    format!("shadowsocks-{version}.x86_64-unknown-linux-gnu.tar.xz")
}

fn download_url(version: &str) -> String {
    DL_URL.to_owned() + "/" + version + "/" + archive_filename(version).as_str()
}

fn write_append(path: &str, contents: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new().append(true).open(path)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

fn is_config_already_modified(conf_path: &str) -> bool {
    fs::read_to_string(conf_path)
        .unwrap_or_default()
        .contains(CONFIGS_CHECK_HEADER)
}

// install logic

fn check_requirements(sh: &Shell) -> Result<()> {
    println!("[prepare] checking requirements");
    let bin_reqs = vec![
        "wget",
        "sha256sum",
        "tar",
        "systemctl",
        "cp",
        "sysctl",
        "ufw",
    ];
    for r in bin_reqs {
        cmd!(sh, "which {r}").quiet().run()?;
    }

    Ok(())
}

fn download(sh: &Shell, install: &Install) -> Result<()> {
    let url = download_url(&install.version);
    cmd!(sh, "wget --no-clobber {url}").run()?;
    cmd!(sh, "wget --no-clobber {url}.sha256").run()?;

    let file = archive_filename(&install.version);
    cmd!(sh, "sha256sum --check {file}.sha256").run()?;

    cmd!(sh, "tar -xf {file}").run()?;
    cmd!(sh, "cp ssservice {SSSERVICE_BIN}").run()?;

    Ok(())
}

fn configure(sh: &Shell, install: &Install) -> Result<()> {
    println!("\n[config] create shadowsocks config");
    let sssconfig = json!({
        "server": "0.0.0.0",
        "server_port": install.server_port,
        "password": install.server_password,
        "method": install.cipher,
    });
    fs::write(CONFIG_FILE, to_string_pretty(&sssconfig)?)?;

    println!("\n[config] create shadowsocks systemd service unit");
    fs::create_dir_all(SYSTEMD_SERVICE_FOLDER)?;
    fs::write(SYSTEMD_SERVICE_FILE, SYSTEMD_SERVICE_TEXT)?;

    cmd!(sh, "systemctl enable ssserver").run()?;
    cmd!(sh, "systemctl restart ssserver").run()?;

    if is_config_already_modified(JOURNALD_CONF) {
        println!("\n[config] tweak log storing policy");
        write_append(JOURNALD_CONF, JOURNALD_CONF_TAIL)?;
    }

    if is_config_already_modified(SYSCTL_CONF) {
        println!("\n[config] tweak kernel");
        write_append(SYSCTL_CONF, SYSCTL_CONF_TAIL)?;
        // apply
        cmd!(sh, "sysctl -p").run()?;
    }

    println!("\n[config] opening ports");
    cmd!(sh, "ufw allow 22").run()?;
    let port = install.server_port.to_string();
    cmd!(sh, "ufw allow {port}").run()?;
    cmd!(sh, "ufw --force enable").run()?;

    Ok(())
}

fn print_config(st: &State) -> Result<()> {
    // this match just for unwrap value, this function will
    // never called with 'Undo' action
    let install = match st.get_install() {
        Some(i) => i,
        None => return Ok(()),
    };

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
        "method": install.cipher,
    });
    let share_url = cmd!(st.sh, "./ssurl -e {CONFIG_FILE}").quiet().read()?;
    println!("####### CLIENT CONFIG #######");
    println!("{}", to_string_pretty(&client_config)?);
    println!("#############################");
    println!("Share URL: {share_url}");
    println!("#############################");

    Ok(())
}

pub fn install(st: &State) -> Result<()> {
    let Action::Install(ref install) = st.action else {
        bail!("wrong action type");
    };

    check_requirements(&st.sh)?;
    download(&st.sh, install)?;
    configure(&st.sh, install)?;
    print_config(st)?;

    cmd!(st.sh, "reboot").run().context("failed to reboot")?;

    Ok(())
}

// undo logic

fn real_undo(st: &State) -> Result<()> {
    cmd!(st.sh, "systemctl disable ssserver").run()?;

    let to_remove = [CONFIG_FILE, SSSERVICE_BIN];
    to_remove.iter().for_each(|f| {
        match fs::remove_file(f) {
            Ok(_) => println!("[undo] remove {f}"),
            Err(e) => eprintln!("Couldn't remove {f}: {e}"),
        };
    });

    Ok(())
}

pub fn undo(st: &State) {
    if let Err(e) = real_undo(st) {
        eprintln!("\nAn error occurred: {e}");
    }
}
