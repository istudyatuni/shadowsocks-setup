use std::process;

use xshell::Shell;

use crate::args::SsType;

pub enum Action {
    Install(Install),
    Undo(Undo),
}

pub struct Undo {
    pub ss_type: SsType,
}

pub struct Install {
    pub ss_type: SsType,
    pub server_port: u32,
    pub server_password: String,
    pub cipher: String,
    pub version: String,
}

pub struct State {
    pub sh: Shell,
    pub action: Action,
}

impl Undo {
    pub fn new(ss_type: SsType) -> Self {
        Self { ss_type }
    }
}

impl Install {
    pub fn new(
        ss_type: SsType,
        server_port: u32,
        server_password: &str,
        cipher: &str,
        version: &str,
    ) -> Self {
        Self {
            ss_type,
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
    pub fn get_install(&self) -> Option<&Install> {
        match &self.action {
            Action::Install(i) => Some(i),
            _ => None,
        }
    }
}
