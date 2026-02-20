use std::error::Error;

use inquire::validator::Validation;

pub fn validate_net_port(value: &u32) -> Result<Validation, Box<dyn Error + Send + Sync>> {
    const MAX_PORT: u32 = (1 << 16) - 1;

    if !matches!(value, 1..=MAX_PORT) {
        return Ok(Validation::Invalid("Port number out of range".into()));
    }

    Ok(Validation::Valid)
}

pub fn validate_empty_string(value: &str) -> Result<Validation, Box<dyn Error + Send + Sync>> {
    if value.is_empty() {
        return Ok(Validation::Invalid("String is empty".into()));
    }

    Ok(Validation::Valid)
}

pub fn validate_simple_email(value: &str) -> Result<Validation, Box<dyn Error + Send + Sync>> {
    if !value.contains("@") && value.len() <= 2 {
        return Ok(Validation::Invalid("String is not an email".into()));
    }

    Ok(Validation::Valid)
}
