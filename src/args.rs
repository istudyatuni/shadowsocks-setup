use std::fmt::Display;

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

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

    /// Do not use directly. Used to separate root/non-root commands
    #[clap(hide = true)]
    InstallStep { step: XrayInstallStep },
}

#[derive(Debug, Default, Serialize, Deserialize, Parser)]
pub struct XrayInstallArgs {
    /// Enable xray api
    #[arg(long)]
    pub api: bool,

    /// Xray api port
    #[arg(long, default_value_t = 47329)]
    pub api_port: u32,

    /// Server domain
    #[arg(long)]
    pub domain: String,

    /// URL to renew domain
    #[arg(long)]
    pub domain_renew_url: Option<String>,

    /// Email for zerossl account
    // todo: check if this is optional
    #[arg(long)]
    pub zerossl_email: String,

    /// Number of new users to add to config. Ignored when --add-user-id is used
    #[arg(long, default_value_t = 1)]
    pub add_users_count: usize,

    /// UUIDs of new users to add to config. Can be repeated or separated with ","
    #[arg(long = "add-user-id", value_delimiter = ',')]
    pub add_user_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum XrayInstallStep {
    DownloadXray,
    InstallXray,
    ConfigureFirewall,
    ConfigureCert,
    ConfigureElse,
}

impl Args {
    pub fn need_root(&self) -> bool {
        match self {
            Self::Xray { cmd } => match cmd {
                XrayArgs::InstallStep { step } => step.need_root(),
                XrayArgs::Install(_) => false,
            },
            _ => false,
        }
    }
}

impl XrayInstallStep {
    const VALUES: &[Self] = &[
        Self::DownloadXray,
        Self::InstallXray,
        Self::ConfigureFirewall,
        Self::ConfigureCert,
        Self::ConfigureElse,
    ];

    pub fn need_root(self) -> bool {
        match self {
            Self::DownloadXray | Self::ConfigureCert => false,
            Self::InstallXray | Self::ConfigureFirewall | Self::ConfigureElse => true,
        }
    }
    pub fn values() -> &'static [Self] {
        assert!(Self::VALUES.len() == Self::value_variants().len());
        Self::VALUES
    }
}

impl Display for XrayInstallStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DownloadXray => "download-xray",
            Self::InstallXray => "install-xray",
            Self::ConfigureFirewall => "configure-firewall",
            Self::ConfigureCert => "configure-cert",
            Self::ConfigureElse => "configure-else",
        };
        s.fmt(f)
    }
}
