use std::fmt::Display;

use clap::{Parser, ValueEnum};

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

    /// Do not use directly. Used to separate root/non-root commands
    #[arg(long, hide = true, default_value_t = XrayInstallStep::DownloadXray)]
    pub next_step: XrayInstallStep,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum XrayInstallStep {
    DownloadXray,
    InstallXray,
}

impl Args {
    pub fn need_root(&self) -> bool {
        match self {
            Self::Xray { cmd } => match cmd {
                XrayArgs::Install(args) => args.next_step.need_root(),
            },
            _ => false,
        }
    }
}

impl XrayInstallStep {
    pub fn need_root(self) -> bool {
        match self {
            Self::DownloadXray => false,
            Self::InstallXray => true,
        }
    }
}

impl Display for XrayInstallStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DownloadXray => "download-xray",
            Self::InstallXray => "install-xray",
        };
        s.fmt(f)
    }
}
