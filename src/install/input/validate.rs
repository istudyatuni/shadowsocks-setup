use std::error::Error;

use inquire::validator::Validation;

pub fn validate_net_port(value: &u32) -> Result<Validation, Box<dyn Error + Send + Sync>> {
    const MAX_PORT: u32 = (1 << 16) - 1;

    if !matches!(value, 1..=MAX_PORT) {
        return Ok(Validation::Invalid("Port number out of range".into()));
    }

    Ok(Validation::Valid)
}
