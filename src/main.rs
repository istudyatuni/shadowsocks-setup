use std::fs::create_dir_all;
use std::process;

use clap::Command;
use state::{Action, Install, State, Undo};

mod args;
mod install;
mod state;

fn prepare_state() -> State {
    let matches =
        args::define_command_line_options(Command::new("Shadowsocks setup")).get_matches();

    match matches.subcommand() {
        Some(("install", subm)) => {
            // this values are required, so can just unwrap
            let ss_type = subm.value_of("TYPE").unwrap();
            let port: i32 = subm.value_of("SERVER_PORT").unwrap().parse().unwrap();
            let pass = subm.value_of("SERVER_PASSWORD").unwrap();
            let cipher = subm.value_of("CIPHER").unwrap();
            let action = Action::Install(Install::new(ss_type, port, pass, cipher));

            State::new(action)
        }
        Some(("undo", subm)) => {
            let ss_type = subm.value_of("TYPE").unwrap();
            let action = Action::Undo(Undo::new(ss_type));
            State::new(action)
        }
        _ => {
            eprintln!("No command");
            process::exit(1);
        }
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
        Action::Install(Install { ss_type, .. }) => match ss_type.as_str() {
            "rust" => install::rust::run(&st),
            "libev" => unimplemented!("libev not implemented"),
            _ => (),
        },
        Action::Undo(Undo { ss_type }) => match ss_type.as_str() {
            "rust" => install::rust::undo(&st),
            "libev" => unimplemented!("libev not implemented"),
            _ => (),
        },
    }
}
