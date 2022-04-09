use std::fs::{create_dir_all, read_to_string, write, OpenOptions};
use std::io::Write;

use pnet::datalink;
use xshell::cmd;

use crate::state::State;

const SS_VERSION: &str = "v1.14.3";
const DL_URL: &str = "https://github.com/shadowsocks/shadowsocks-rust/releases/download";

const BIN_FOLDER: &str = "/usr/local/bin/";
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

fn archive_filename() -> String {
    format!("shadowsocks-{}.x86_64-unknown-linux-gnu.tar.xz", SS_VERSION)
}

fn download_url() -> String {
    DL_URL.to_owned() + "/" + SS_VERSION + "/" + archive_filename().as_str()
}

fn write_append(path: &str, contents: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().append(true).open(path)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

fn is_config_already_modified(conf_path: &str) -> bool {
    match read_to_string(conf_path)
        .unwrap_or_default()
        .find(CONFIGS_CHECK_HEADER)
    {
        Some(_) => true,
        None => false,
    }
}

// logic

fn download(st: &State) -> Result<(), Box<dyn std::error::Error>> {
    let url = download_url();
    cmd!(st.sh, "wget --no-clobber {url}").run()?;
    cmd!(st.sh, "wget --no-clobber {url}.sha256").run()?;

    let file = archive_filename();
    cmd!(st.sh, "sha256sum --check {file}.sha256").run()?;

    cmd!(st.sh, "tar -xf {file}").run()?;
    cmd!(st.sh, "mv ssservice {BIN_FOLDER}").run()?;

    Ok(())
}

fn configure(st: &State) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n[config] create shadowsocks config");
    let sssconfig = format!(
        r#"{{
    "server": "0.0.0.0",
    "server_port": {},
    "password": "{}",
    "method": "{}"
}}"#,
        st.server_port, st.server_password, st.cipher
    );
    write(CONFIG_FILE, sssconfig)?;

    println!("\n[config] create shadowsocks systemd service unit");
    create_dir_all(SYSTEMD_SERVICE_FOLDER)?;
    write(SYSTEMD_SERVICE_FILE, SYSTEMD_SERVICE_TEXT)?;

    cmd!(st.sh, "systemctl enable ssserver").run()?;
    cmd!(st.sh, "systemctl restart ssserver").run()?;

    if is_config_already_modified(JOURNALD_CONF) {
        println!("\n[config] tweak log storing policy");
        write_append(JOURNALD_CONF, JOURNALD_CONF_TAIL)?;
    }

    if is_config_already_modified(SYSCTL_CONF) {
        println!("\n[config] tweak kernel");
        write_append(SYSCTL_CONF, SYSCTL_CONF_TAIL)?;
        // apply
        cmd!(st.sh, "sysctl -p").run()?;
    }

    println!("\n[config] opening ports");
    cmd!(st.sh, "ufw allow 22").run()?;
    let port = st.server_port.clone();
    cmd!(st.sh, "ufw allow {port}").run()?;
    cmd!(st.sh, "ufw --force enable").run()?;

    Ok(())
}

fn print_config(st: &State) -> Result<(), Box<dyn std::error::Error>> {
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

    println!(
        r#"####### CLIENT CONFIG #######
{{
    "server": "{}",
    "server_port": {},
    "local_port": 1080,
    "password": "{}",
    "method": "{}"
}}
#############################
Share URL:"#,
        server_ip, st.server_port, st.server_password, st.cipher
    );
    cmd!(st.sh, "./ssurl -e {CONFIG_FILE}").quiet().run()?;

    Ok(())
}

pub fn run(st: &State) {
    if let Err(e) = download(&st) {
        eprintln!("\nAn error occurred when downloading: {e}");
    }
    if let Err(e) = configure(&st) {
        eprintln!("\nAn error occurred when configuring: {e}");
    }
    if let Err(e) = print_config(&st) {
        eprintln!("\nAn error occurred: {e}");
    }
}
