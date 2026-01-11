use std::{ops::Deref, str::FromStr};

#[derive(Debug, Clone)]
pub struct Version(String);

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl FromStr for Version {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('v');
        let parts: Result<Vec<u32>, _> = s.split('.').map(|v| v.parse()).collect();
        if parts.is_err() {
            return Err("invalid number part in version");
        }

        Ok(Self(s.to_string()))
    }
}

impl Deref for Version {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}
