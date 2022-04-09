use std::error::Error;

use xshell::Shell;

#[derive(Debug)]
pub struct State {
    pub sh: Shell,
    pub server_port: String,
    pub server_password: String,
    pub cipher: String,
}

impl State {
    pub fn new(
        server_port: &str,
        server_password: &str,
        cipher: &str,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            sh: Shell::new()?,
            server_port: server_port.to_string(),
            server_password: server_password.to_string(),
            cipher: cipher.to_string(),
        })
    }
}
