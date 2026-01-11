use crate::version::Version;

pub fn validate_net_port(value: &u32) -> Result<(), &'static str> {
    const MAX_PORT: u32 = (1 << 16) - 1;

    if !matches!(value, 1..=MAX_PORT) {
        return Err("port number out of range");
    }
    Ok(())
}

#[expect(clippy::ptr_arg)]
pub fn validate_version(value: &String) -> Result<(), &'static str> {
    let _: Version = value.parse().map_err(|_| "invalid version format")?;
    Ok(())
}
