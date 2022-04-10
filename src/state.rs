use std::process;

use xshell::Shell;

pub enum Action {
    Install(Install),
    Undo(Undo),
}

pub struct Undo {
    pub ss_type: String,
}

pub struct Install {
    pub ss_type: String,
    pub server_port: String,
    pub server_password: String,
    pub cipher: String,
}

pub struct State {
    pub sh: Shell,
    pub action: Action,
}

impl Undo {
    pub fn new(ss_type: &str) -> Self {
        Self {
            ss_type: ss_type.to_string(),
        }
    }
}

impl Install {
    pub fn new(ss_type: &str, server_port: &str, server_password: &str, cipher: &str) -> Self {
        Self {
            ss_type: ss_type.to_string(),
            server_port: server_port.to_string(),
            server_password: server_password.to_string(),
            cipher: cipher.to_string(),
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
    pub fn get_install(&self) -> Option<&Install> {
        match &self.action {
            Action::Install(i) => Some(i),
            _ => None,
        }
    }
}
