use clap::ValueEnum;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::{
    args::{Cipher, InstallArgs},
    version::Version,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Install {
    pub server_port: u32,
    pub server_password: String,
    pub cipher: String,
    pub version: Version,
}

impl Install {
    pub fn ask(
        args: InstallArgs,
        installed_version: Option<&str>,
        latest_version: &str,
    ) -> Result<Self> {
        let mut asker = InstallInput::from_args(args);
        asker.ask_version(latest_version)?;

        if let Some(version) = installed_version {
            let version = version.trim_start_matches('v');
            if let Some(input_version) = &asker.version {
                if version == input_version.trim_start_matches('v')
                    && Confirm::new()
                        .with_prompt("Shadowsocks v{version} already installed, continue?")
                        .show_default(false)
                        .interact()?
                {
                    return Err(Error::Aborted);
                }
            }
        }

        asker.ask_server_port()?;
        asker.ask_server_password()?;
        asker.ask_cipher()?;

        Ok(Install {
            server_port: asker.server_port.expect("should be asked"),
            server_password: asker.server_password.expect("should be asked"),
            cipher: asker.cipher.expect("should be asked"),
            version: asker.version.expect("should be asked"),
        })
    }
}

#[derive(Debug, Default)]
struct InstallInput {
    server_port: Option<u32>,
    server_password: Option<String>,
    cipher: Option<String>,
    version: Option<Version>,
}

impl InstallInput {
    fn from_args(args: InstallArgs) -> Self {
        Self {
            server_port: args.port,
            server_password: args.password,
            cipher: args.cipher.map(|c| c.to_string()),
            version: args.version,
        }
    }
    fn ask_server_port(&mut self) -> Result<()> {
        if self.server_port.is_some() {
            return Ok(());
        }

        self.server_port = Some(
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Server port")
                .validate_with(super::validate::validate_net_port)
                .interact_text()?,
        );
        Ok(())
    }
    fn ask_server_password(&mut self) -> Result<()> {
        if self.server_password.is_some() {
            return Ok(());
        }

        self.server_password = Some(
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Server password")
                .interact_text()?,
        );
        Ok(())
    }
    fn ask_cipher(&mut self) -> Result<()> {
        if self.cipher.is_some() {
            return Ok(());
        }

        let items: Vec<_> = Cipher::value_variants()
            .iter()
            .map(|v| v.to_string())
            .collect();

        let selected = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Cipher")
            .items(&items)
            .default(0)
            .interact()?;

        self.cipher = Some(items[selected].to_string());
        Ok(())
    }
    fn ask_version(&mut self, latest_version: &str) -> Result<()> {
        if self.version.is_some() {
            return Ok(());
        }

        self.version = Some(
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Shadowsocks version")
                .with_initial_text(latest_version)
                .validate_with(super::validate::validate_version)
                .interact_text()?
                .parse()
                .expect("should be validated"),
        );
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("aborted")]
    Aborted,
    #[error("{0}")]
    Dialog(#[from] dialoguer::Error),
}
