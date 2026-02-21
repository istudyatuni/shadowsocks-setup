use clap::ValueEnum;
use inquire::{Confirm, CustomType, Select, Text};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{args::ShadowsocksInstallArgs, cipher::Cipher, update_from_options, version::Version};

use super::SerializableState;

pub type Result<T, E = Error> = std::result::Result<T, E>;

const TEMP_PATH: &str = "/tmp/ssserver-install-shadosocks-input-state.json";

#[derive(Debug)]
pub struct Install {
    pub server_port: u32,
    pub server_password: String,
    pub cipher: Cipher,
    pub version: Version,
}

impl Install {
    pub fn ask(
        args: ShadowsocksInstallArgs,
        installed_version: Option<Version>,
        latest_version: Version,
    ) -> Result<Self> {
        let mut asker = match DataInput::load_state() {
            Ok(a) => a.update_from_args(args),
            Err(e) => {
                error!("failed to load input state: {e}");
                DataInput::default().update_from_args(args)
            }
        };
        asker.ask_version(latest_version)?;

        if let Some(version) = installed_version
            && let Some(input_version) = &asker.version
            && version == *input_version
            && Confirm::new("Shadowsocks v{version} already installed, continue?")
                .with_default(false)
                .prompt()?
        {
            return Err(Error::Aborted);
        }

        asker.ask_server_port()?;
        asker.ask_server_password()?;
        asker.ask_cipher()?;

        if let Err(e) = DataInput::clean_state() {
            error!("failed to cleanup input state: {e}");
        }

        Ok(Install {
            server_port: asker.server_port.expect("should be asked"),
            server_password: asker.server_password.expect("should be asked"),
            cipher: asker.cipher.expect("should be asked"),
            version: asker.version.expect("should be asked"),
        })
    }
}

#[derive(Debug)]
pub struct Update {
    pub version: Version,
}

impl Update {
    pub fn ask(latest_version: Version) -> Result<Self> {
        let mut asker = DataInput::default();
        asker.ask_version(latest_version)?;

        Ok(Self {
            version: asker.version.expect("should be asked"),
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DataInput {
    server_port: Option<u32>,
    server_password: Option<String>,
    cipher: Option<Cipher>,
    version: Option<Version>,
}

impl SerializableState for DataInput {
    const TEMP_PATH: &str = TEMP_PATH;
}

impl DataInput {
    fn update_from_args(mut self, args: ShadowsocksInstallArgs) -> Self {
        update_from_options!(
            self.server_port => args.port,
            self.server_password => args.password,
            self.cipher => args.cipher,
            self.version => args.version,
        );
        self
    }
    fn ask_server_port(&mut self) -> Result<()> {
        self.server_port = Some(
            CustomType::<u32>::new("Server port")
                .with_starting_input(
                    self.server_port
                        .map(|p| ToString::to_string(&p))
                        .unwrap_or_default()
                        .as_str(),
                )
                .with_error_message("Invalid number")
                .with_validator(super::validate::validate_net_port)
                .prompt()?,
        );
        self.save_state();
        Ok(())
    }
    fn ask_server_password(&mut self) -> Result<()> {
        self.server_password = Some(
            Text::new("Server password")
                .with_initial_value(self.server_password.as_deref().unwrap_or_default())
                .prompt()?,
        );
        self.save_state();
        Ok(())
    }
    fn ask_cipher(&mut self) -> Result<()> {
        let items = Cipher::value_variants();
        let start = if let Some(cipher) = self.cipher {
            items.iter().position(|&i| i == cipher).unwrap_or_default()
        } else {
            0
        };
        let cipher = Select::new("Cipher", items.to_vec())
            .with_starting_cursor(start)
            .prompt()?;

        self.cipher = Some(cipher);
        self.save_state();
        Ok(())
    }
    fn ask_version(&mut self, latest_version: Version) -> Result<()> {
        self.version = Some(
            CustomType::<Version>::new("Shadowsocks version")
                .with_starting_input(
                    self.version
                        .as_ref()
                        .unwrap_or(&latest_version)
                        .as_prefixed()
                        .as_str(),
                )
                .prompt()?,
        );
        self.save_state();
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("aborted")]
    Aborted,

    #[error("{0}")]
    Inquire(#[from] inquire::error::InquireError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}
