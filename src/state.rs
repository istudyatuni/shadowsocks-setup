use std::process;

use xshell::Shell;

pub enum Action {
    Install(Install),
    Undo,
}

pub struct Install {
    pub server_port: u32,
    pub server_password: String,
    pub cipher: String,
    pub version: String,
}

pub struct State {
    pub sh: Shell,
    pub action: Action,
}

impl Install {
    pub fn new(server_port: u32, server_password: &str, cipher: &str, version: &str) -> Self {
        Self {
            server_port,
            server_password: server_password.to_string(),
            cipher: cipher.to_string(),
            version: version.to_string(),
        }
    }
}

impl State {
    pub fn new(action: Action) -> Self {
        let sh = Shell::new().unwrap_or_else(|e| {
            println!("Couldn't init shell: {e}");
            process::exit(1)
        });
        Self { sh, action }
    }
}
