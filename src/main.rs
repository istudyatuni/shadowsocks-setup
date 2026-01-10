use std::fs::create_dir_all;
use std::process;

use args::{Args, SsType};
use clap::Parser;
use state::{Action, Install, State, Undo};

mod args;
mod install;
mod state;

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

fn main() {
    let st = prepare_state();

    // disable in dev build
    if cfg!(not(debug_assertions)) && sudo::check() != sudo::RunningAs::Root {
        eprintln!("This script requires sudo");
        process::exit(1);
    }

    const ARTIFACTS_DIR: &str = "shadowsocks-artifacts";
    create_dir_all(ARTIFACTS_DIR).unwrap_or_else(|e| {
        eprintln!("Couldn't create directory: {e}");
        process::exit(1);
    });
    st.sh.change_dir(ARTIFACTS_DIR);

    match &st.action {
        Action::Install(Install { ss_type, .. }) => match ss_type {
            SsType::Rust => install::rust::install(&st),
            SsType::Libev => eprintln!("libev is not implemented"),
        },
        Action::Undo(Undo { ss_type }) => match ss_type {
            SsType::Rust => install::rust::undo(&st),
            SsType::Libev => eprintln!("libev is not implemented"),
        },
    }
}
