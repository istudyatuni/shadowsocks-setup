use std::fs::create_dir_all;

use anyhow::{bail, Context, Result};
use clap::Parser;

use args::{Args, SsType};
use github::get_latest_release_tag;
use state::{Action, Install, State, Undo};

mod args;
mod github;
mod install;
mod state;

const ARTIFACTS_DIR: &str = "shadowsocks-artifacts";

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let st = prepare_state()?;

    // disable in dev build
    if cfg!(not(debug_assertions)) && sudo::check() != sudo::RunningAs::Root {
        bail!("This script requires sudo");
    }

    create_dir_all(ARTIFACTS_DIR).context("failed to create artifacts dir")?;
    st.sh.change_dir(ARTIFACTS_DIR);
    std::env::set_current_dir(ARTIFACTS_DIR).context("failed to change current dir")?;

    match &st.action {
        Action::Install(install @ Install { ss_type, .. }) => match ss_type {
            SsType::Rust => install::rust::install(&st.sh, install)?,
            SsType::Libev => bail!("libev is not implemented"),
        },
        Action::Undo(Undo { ss_type }) => match ss_type {
            SsType::Rust => install::rust::undo(&st.sh)?,
            SsType::Libev => bail!("libev is not implemented"),
        },
    }

    Ok(())
}

fn get_ss_version(ss_type: SsType, provided: Option<&str>) -> Result<String> {
    if let Some(version) = provided {
        return Ok(format!("v{version}"));
    }
    let (owner, repo) = match ss_type {
        SsType::Rust => ("shadowsocks", "shadowsocks-rust"),
        SsType::Libev => ("shadowsocks", "shadowsocks-libev"),
    };
    get_latest_release_tag(owner, repo).context("failed to get latest release")
}

fn prepare_state() -> Result<State> {
    let args = Args::parse();

    let st = match args {
        Args::Install {
            ty,
            port,
            password,
            cipher,
            version,
        } => {
            let version = get_ss_version(ty, version.as_deref())?;
            State::new(Action::Install(Install::new(
                ty,
                port,
                &password,
                &cipher.to_string(),
                &version,
            )))
        }
        Args::Undo { ty } => State::new(Action::Undo(Undo::new(ty))),
    };
    Ok(st)
}
