use std::fs::create_dir_all;

use anyhow::{bail, Context, Result};
use args::{Args, SsType};
use clap::Parser;
use state::{Action, Install, State, Undo};

mod args;
mod install;
mod state;

const ARTIFACTS_DIR: &str = "shadowsocks-artifacts";

fn prepare_state() -> State {
    let args = Args::parse();

    match args {
        Args::Install {
            ty,
            port,
            password,
            cipher,
            version,
        } => State::new(Action::Install(Install::new(
            ty,
            port,
            &password,
            &cipher.to_string(),
            &version,
        ))),
        Args::Undo { ty } => State::new(Action::Undo(Undo::new(ty))),
    }
}

fn main() -> Result<()> {
    let st = prepare_state();

    // disable in dev build
    if cfg!(not(debug_assertions)) && sudo::check() != sudo::RunningAs::Root {
        bail!("This script requires sudo");
    }

    create_dir_all(ARTIFACTS_DIR).context("failed to create artifacts dir")?;
    st.sh.change_dir(ARTIFACTS_DIR);

    match &st.action {
        Action::Install(Install { ss_type, .. }) => match ss_type {
            SsType::Rust => install::rust::install(&st),
            SsType::Libev => bail!("libev is not implemented"),
        },
        Action::Undo(Undo { ss_type }) => match ss_type {
            SsType::Rust => install::rust::undo(&st),
            SsType::Libev => bail!("libev is not implemented"),
        },
    }

    Ok(())
}
