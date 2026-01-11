use std::fmt::Display;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
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
        let Some(value) = self.to_possible_value() else {
            return "-".fmt(f);
        };
        value.get_name().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_cipher_fmt() {
        use Cipher::*;

        assert_eq!(Aes256Gcm.to_string(), "aes-256-gcm");
        assert_eq!(Chacha20IetfPoly1305.to_string(), "chacha20-ietf-poly1305");
        assert_eq!(Aes128Gcm.to_string(), "aes-128-gcm");

        assert_eq!(
            serde_json::to_string(&json!({ "value": Cipher::Aes128Gcm })).unwrap(),
            "{\"value\":\"Aes128Gcm\"}"
        );
    }
}
