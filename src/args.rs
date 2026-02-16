use clap::Parser;

use crate::{cipher::Cipher, version::Version};

/// Shadowsocks setup
#[derive(Debug, Parser)]
#[clap(version)]
pub enum Args {
    Shadowsocks {
        #[clap(subcommand)]
        cmd: ShadowsocksArgs,
    },
    Xray {
        #[clap(subcommand)]
        cmd: XrayArgs,
    },
}

/// Shadowsocks setup
#[derive(Debug, Parser)]
pub enum ShadowsocksArgs {
    /// Install shadowsocks
    Install(ShadowsocksInstallArgs),
    /// Update shadowsocks
    Update(ShadowsocksUpdateArgs),
    /// Uninstall shadowsocks
    Uninstall,
}

#[derive(Debug, Parser)]
pub struct ShadowsocksInstallArgs {
    /// Server port
    #[arg(long)]
    pub port: Option<u32>,

    /// Server password
    #[arg(long)]
    pub password: Option<String>,

    /// AEAD cipher
    #[arg(long)]
    pub cipher: Option<Cipher>,

    /// Shadowsocks version to install
    #[arg(long)]
    pub version: Option<Version>,
}

#[derive(Debug, Parser)]
pub struct ShadowsocksUpdateArgs {
    /// Shadowsocks version to install
    #[arg(long)]
    pub version: Option<Version>,
}

/// Xray setup
#[derive(Debug, Parser)]
pub enum XrayArgs {
    /// Install xray
    Install(XrayInstallArgs),
}

#[derive(Debug, Parser)]
pub struct XrayInstallArgs {
    /// Enable xray api
    #[arg(long, default_value_t = false)]
    pub api: bool,

    /// Xray api port
    #[arg(long)]
    pub api_port: Option<u32>,

    /// Server domain
    #[arg(long)]
    pub domain: Option<String>,

    /// URL to renew domain
    #[arg(long)]
    pub domain_renew_url: Option<String>,
}
