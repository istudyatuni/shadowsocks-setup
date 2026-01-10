use std::{fmt::Display, ops::Deref, str::FromStr};

use clap::{Parser, ValueEnum};

/// Shadowsocks setup
#[derive(Debug, Parser)]
#[clap(version)]
pub enum Args {
    /// Install shadowsocks
    Install {
        /// Server port
        #[arg(long)]
        port: u32,

        /// Server password
        #[arg(long)]
        password: String,

        /// AEAD cipher
        #[arg(long, default_value_t = Cipher::Aes256Gcm)]
        cipher: Cipher,

        /// Shadowsocks version to install
        #[arg(long)]
        version: Option<Version>,
    },
    /// Uninstall shadowsocks
    Undo,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Cipher {
    #[value(name = "aes-256-gcm")]
    Aes256Gcm,
    #[value(name = "chacha20-ietf-poly1305")]
    Chacha20IetfPoly1305,
    #[value(name = "aes-128-gcm")]
    Aes128Gcm,
}

impl Display for Cipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Cipher::Aes256Gcm => "aes-256-gcm",
            Cipher::Chacha20IetfPoly1305 => "chacha20-ietf-poly1305",
            Cipher::Aes128Gcm => "aes-128-gcm",
        };
        s.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct Version(String);

impl FromStr for Version {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('v');
        let parts: Result<Vec<u32>, _> = s.split('.').map(|v| v.parse()).collect();
        if parts.is_err() {
            return Err("invalid number part in version");
        }

        Ok(Self(s.to_string()))
    }
}

impl Deref for Version {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}
