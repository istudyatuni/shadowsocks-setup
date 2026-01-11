use std::net::IpAddr;

use anyhow::{Context, Result};
use pnet::datalink;

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
