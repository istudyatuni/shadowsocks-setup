use std::fmt::Display;

use clap::{Parser, ValueEnum};

/// Shadowsocks setup
#[derive(Debug, Parser)]
#[clap(version)]
pub enum Args {
    /// Install shadowsocks
    Install {
        /// Shadowsocks installation type
        #[arg(long = "type", default_value = "rust", id = "TYPE")]
        ty: SsType,

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
        version: Option<String>,
    },
    /// Install shadowsocks
    Undo {
        /// Shadowsocks installation type
        #[arg(long = "type", default_value = "rust", id = "TYPE")]
        ty: SsType,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SsType {
    Rust,
    #[value(skip)]
    #[expect(unused)]
    Libev,
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
