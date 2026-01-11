use clap::ValueEnum;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::{args::InstallArgs, cipher::Cipher, version::Version};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Install {
    pub server_port: u32,
    pub server_password: String,
    pub cipher: Cipher,
    pub version: Version,
}

impl Install {
    pub fn ask(
        args: InstallArgs,
        installed_version: Option<Version>,
        latest_version: Version,
    ) -> Result<Self> {
        let mut asker = InstallInput::from_args(args);
        asker.ask_version(latest_version)?;

        if let Some(version) = installed_version {
            if let Some(input_version) = &asker.version {
                if version == *input_version
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
    cipher: Option<Cipher>,
    version: Option<Version>,
}

impl InstallInput {
    fn from_args(args: InstallArgs) -> Self {
        Self {
            server_port: args.port,
            server_password: args.password,
            cipher: args.cipher,
            version: args.version,
        }
    }
    fn ask_server_port(&mut self) -> Result<()> {
        self.server_port = Some(
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Server port")
                .with_initial_text(
                    self.server_port
                        .map(|p| ToString::to_string(&p))
                        .unwrap_or_default(),
                )
                .validate_with(super::validate::validate_net_port)
                .interact_text()?,
        );
        Ok(())
    }
    fn ask_server_password(&mut self) -> Result<()> {
        self.server_password = Some(
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Server password")
                .with_initial_text(self.server_password.as_deref().unwrap_or_default())
                .interact_text()?,
        );
        Ok(())
    }
    fn ask_cipher(&mut self) -> Result<()> {
        if self.cipher.is_some() {
            return Ok(());
        }

        let items = Cipher::value_variants();
        let selected = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Cipher")
            .items(items)
            .default(0)
            .interact()?;

        self.cipher = Some(items[selected]);
        Ok(())
    }
    fn ask_version(&mut self, latest_version: Version) -> Result<()> {
        self.version = Some(
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Shadowsocks version")
                .with_initial_text(
                    self.version
                        .as_ref()
                        .unwrap_or(&latest_version)
                        .as_prefixed(),
                )
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
