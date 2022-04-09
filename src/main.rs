use std::process;
use std::fs::create_dir_all;

use clap::Command;
use state::State;

mod args;
mod install;
mod state;

fn main() {
    let matches =
        args::define_command_line_options(Command::new("Shadowsocks setup")).get_matches();

    let st = State::new(
        // this values are required
        matches.value_of("SERVER_PORT").unwrap(),
        matches.value_of("SERVER_PASSWORD").unwrap(),
        matches.value_of("CIPHER").unwrap(),
    )
    .unwrap_or_else(|e| {
        println!("Cannot init shell: {e}");
        process::exit(1)
    });

    let _: i32 = st.server_port.parse().unwrap_or_else(|_| {
        eprintln!("Port shold be a number");
        process::exit(1);
    });

    const ARTIFACTS_DIR: &str = "shadowsocks-artifacts";
    create_dir_all(ARTIFACTS_DIR).unwrap_or_else(|e| {
        eprintln!("Couldn't create directory: {e}");
        process::exit(1);
    });
    st.sh.change_dir(ARTIFACTS_DIR);

    if let Some(install) = matches.value_of("INSTALL_TYPE") {
        match install {
            "rust" => install::rust::run(&st),
            "libev" => unimplemented!("libev not implemented"),
            _ => (),
        }
    }
}
