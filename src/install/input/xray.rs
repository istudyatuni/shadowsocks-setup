use inquire::{Confirm, CustomType, Editor, Text};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{args::XrayInstallArgs, update_from_options};

use super::SerializableState;

pub type Result<T, E = Error> = std::result::Result<T, E>;

const TEMP_PATH: &str = "/tmp/ssserver-install-shadosocks-input-state.json";
const ADD_USERS_DEFAULT_FILE: &str = r##"
# Place each UUID on its own line, e.g.
# af068bb5-ec48-46ff-bdc4-80344bb5f5c7
# 6aa483d1-ada6-41e9-a048-3b868631ebc7
# Lines starting with "#" are ignored
"##;

#[derive(Debug, Serialize, Deserialize)]
pub struct Install {
    pub api: bool,
    pub api_port: u32,
    pub domain: String,
    pub domain_renew_url: Option<String>,
    pub zerossl_email: String,
    pub add_users_count: usize,
    pub add_user_ids: Vec<String>,
}

impl Install {
    pub fn ask(args: XrayInstallArgs) -> Result<Self> {
        let mut asker = match DataInput::load_state() {
            Ok(a) => a.update_from_args(args),
            Err(e) => {
                error!("failed to load input state: {e}");
                DataInput::default().update_from_args(args)
            }
        };

        asker.ask_api()?;
        if asker.api {
            asker.ask_api_port()?;
        }
        asker.ask_domain()?;
        asker.ask_domain_renew_url()?;
        asker.ask_zerossl_email()?;
        // should be before ask_add_users_count
        asker.ask_add_users_ids()?;
        if asker.add_user_ids.is_empty() {
            asker.ask_add_users_count()?;
        }

        let res = Install {
            api: asker.api,
            api_port: asker.api_port,
            domain: asker.domain.expect("should be asked"),
            domain_renew_url: asker.domain_renew_url,
            zerossl_email: asker.zerossl_email.expect("should be asked"),
            add_users_count: asker.add_users_count,
            add_user_ids: asker.add_user_ids,
        };

        if let Err(e) = DataInput::clean_state() {
            error!("failed to cleanup input state: {e}");
        }

        Ok(res)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DataInput {
    api: bool,
    api_port: u32,
    domain: Option<String>,
    domain_renew_url: Option<String>,
    zerossl_email: Option<String>,
    add_users_count: usize,
    add_user_ids: Vec<String>,
}

impl SerializableState for DataInput {
    const TEMP_PATH: &str = TEMP_PATH;
}

impl DataInput {
    fn update_from_args(mut self, args: XrayInstallArgs) -> Self {
        self.api = args.api;
        self.api_port = args.api_port;
        self.add_users_count = args.add_users_count;
        self.add_user_ids = args.add_user_ids;
        update_from_options!(
            self.domain => args.domain,
            self.domain_renew_url => args.domain_renew_url,
            self.zerossl_email => args.zerossl_email,
        );

        self
    }
    fn ask_api(&mut self) -> Result<()> {
        self.api = Confirm::new("Enable API?")
            .with_default(self.api)
            .prompt()?;
        self.save_state();
        Ok(())
    }
    fn ask_api_port(&mut self) -> Result<()> {
        self.api_port = CustomType::<u32>::new("API port")
            .with_starting_input(&self.api_port.to_string())
            .with_error_message("Invalid number")
            .with_validator(super::validate::validate_net_port)
            .prompt()?;
        self.save_state();
        Ok(())
    }
    fn ask_domain(&mut self) -> Result<()> {
        self.domain = Some(
            Text::new("Domain")
                .with_initial_value(self.domain.as_deref().unwrap_or_default())
                .with_validator(super::validate::validate_empty_string)
                .prompt()?,
        );
        self.save_state();
        Ok(())
    }
    fn ask_domain_renew_url(&mut self) -> Result<()> {
        let res = Text::new("Domain renew URL")
            .with_initial_value(self.domain_renew_url.as_deref().unwrap_or_default())
            .prompt()?;
        if !res.is_empty() {
            self.domain_renew_url = Some(res);
        } else {
            self.domain_renew_url = None;
        }
        self.save_state();
        Ok(())
    }
    fn ask_zerossl_email(&mut self) -> Result<()> {
        self.zerossl_email = Some(
            Text::new("ZeroSSL email")
                .with_initial_value(self.zerossl_email.as_deref().unwrap_or_default())
                .with_validator(super::validate::validate_empty_string)
                .with_validator(super::validate::validate_simple_email)
                .prompt()?,
        );
        self.save_state();
        Ok(())
    }
    fn ask_add_users_count(&mut self) -> Result<()> {
        self.add_users_count = CustomType::<usize>::new("How many users to add")
            .with_starting_input(&self.add_users_count.to_string())
            .with_error_message("Invalid number")
            .prompt()?;
        self.save_state();
        Ok(())
    }
    fn ask_add_users_ids(&mut self) -> Result<()> {
        let add_users = Confirm::new("Add users by uuid? This will open an editor")
            .with_help_message("You can skip this step and set how many users to add later")
            .with_default(false)
            .prompt()?;
        if !add_users {
            return Ok(());
        }
        let text = Editor::new("Add users")
            .with_predefined_text(ADD_USERS_DEFAULT_FILE.trim_start())
            .prompt()?;
        self.add_user_ids = parse_add_users_file(&text);
        self.save_state();
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Inquire(#[from] inquire::error::InquireError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

fn parse_add_users_file(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.starts_with("#"))
        .map(ToString::to_string)
        .collect()
}
