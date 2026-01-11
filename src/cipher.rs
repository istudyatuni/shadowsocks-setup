use std::fmt::Display;

use clap::ValueEnum;

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
