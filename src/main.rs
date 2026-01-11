use std::fs::create_dir_all;

use anyhow::{bail, Context, Result};
use clap::Parser;
use xshell::Shell;

use args::Args;
use github::get_latest_release_tag;
use state::{Action, Install, State};

mod args;
mod github;
mod install;

const ARTIFACTS_DIR: &str = "shadowsocks-artifacts";

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let args = Args::parse();
    let sh = Shell::new()?;

    // disable in dev build
    if cfg!(not(debug_assertions)) && sudo::check() != sudo::RunningAs::Root {
        bail!("This script requires sudo");
    }

    create_dir_all(ARTIFACTS_DIR).context("failed to create artifacts dir")?;
    sh.change_dir(ARTIFACTS_DIR);
    std::env::set_current_dir(ARTIFACTS_DIR).context("failed to change current dir")?;

    match args {
        Args::Install {
            port,
            password,
            cipher,
            version,
        } => {
            let version = get_ss_version(version.as_deref())?;
            let install = Install {
                server_port: port,
                server_password: password,
                cipher: cipher.to_string(),
                version,
            };
            install::shadowsocks::install(&sh, &install)?
        }
        Args::Undo => install::shadowsocks::undo(&sh)?,
    }

    Ok(())
}

fn get_ss_version(provided: Option<&str>) -> Result<String> {
    if let Some(version) = provided {
        return Ok(format!("v{version}"));
    }
    get_latest_release_tag("shadowsocks", "shadowsocks-rust")
        .context("failed to get latest release")
}
