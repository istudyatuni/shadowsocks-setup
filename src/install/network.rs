use std::net::IpAddr;

use anyhow::{Context, Result};
use pnet::datalink;
use tracing::debug;
use xshell::{Shell, cmd};

pub fn get_ipv4() -> Result<IpAddr> {
    let all_interfaces = datalink::interfaces();
    let default_interface = all_interfaces
        .iter()
        .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty())
        .context("failed to find ip address")?;
    let server_ip = default_interface
        .ips
        .iter()
        .find(|e| e.is_ipv4())
        .context("failed to find ipv4 address")?
        .ip();
    Ok(server_ip)
}

pub fn open_firewall_ports_and_enable(sh: &Shell, ports: &[u32]) -> Result<()> {
    debug!("opening firewall ports");

    for port in ports {
        let port = port.to_string();
        cmd!(sh, "ufw allow {port}").run()?;
    }
    cmd!(sh, "ufw --force enable").run()?;

    Ok(())
}
