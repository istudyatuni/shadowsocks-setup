use std::process;

use clap::Command;
use state::State;

mod args;
mod install;
mod state;

fn main() {
    let matches =
        args::define_command_line_options(Command::new("Shadowsocks setup")).get_matches();

    let sh = State::new(
        // this values are required
        matches.value_of("SERVER_PORT").unwrap(),
        matches.value_of("SERVER_PASSWORD").unwrap(),
        matches.value_of("CIPHER").unwrap(),
    )
    .unwrap_or_else(|e| {
        println!("Cannot init shell: {e}");
        process::exit(1)
    });

    let _: i32 = sh.server_port.parse().unwrap_or_else(|_| {
        eprintln!("Port shold be a number");
        process::exit(1);
    });

    if let Some(install) = matches.value_of("INSTALL_TYPE") {
        match install {
            "rust" => install::rust::run(&sh),
            "libev" => unimplemented!("libev not implemented"),
            _ => (),
        }
    }
}
