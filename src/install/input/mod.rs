use std::path::PathBuf;

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use tracing::error;

pub mod shadowsocks;
mod validate;
pub mod xray;

/// `()` are used to use `=` instead of `=>`
#[macro_export]
macro_rules! update_from_options {
    ($(($to:expr) = $from:expr),* $(,)?) => {$(
        if let arg @ Some(_) = $from {
            $to = arg;
        }
    )*};
}

trait SerializableState
where
    Self: Default + Serialize + DeserializeOwned,
{
    const TEMP_PATH: &str;

    fn load_state() -> Result<Self> {
        let path = PathBuf::from(Self::TEMP_PATH);
        if !path.exists() {
            return Ok(Self::default());
        }

        let s = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&s)?)
    }

    fn save_state(&self) {
        let save = || -> Result<()> {
            let s = serde_json::to_string_pretty(self)?;
            std::fs::write(Self::TEMP_PATH, s)?;
            Ok(())
        };
        if let Err(e) = save() {
            error!("failed to save input state: {e}");
        }
    }

    fn clean_state() -> Result<()> {
        std::fs::remove_file(Self::TEMP_PATH)?;
        Ok(())
    }
}
