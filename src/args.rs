use clap::Parser;

use crate::{cipher::Cipher, version::Version};

/// Shadowsocks setup
#[derive(Debug, Parser)]
#[clap(version)]
pub enum Args {
    /// Install shadowsocks
    Install(InstallArgs),
    /// Update shadowsocks
    Update(UpdateArgs),
    /// Uninstall shadowsocks
    Uninstall,
}

#[derive(Debug, Parser)]
pub struct InstallArgs {
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
pub struct UpdateArgs {
    /// Shadowsocks version to install
    #[arg(long)]
    pub version: Option<Version>,
}
