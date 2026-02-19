use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use tracing::{debug, error};
use xshell::{Shell, cmd};

pub mod input;
mod network;
pub mod shadowsocks;
pub mod xray;
pub mod xray_config;

const ARTIFACTS_DIR: &str = "artifacts";

pub fn check_requirements(sh: &Shell, bin_reqs: &[&str]) -> Result<()> {
    debug!("checking required executables");
    let mut missed = false;
    for r in bin_reqs {
        if !exe_in_path(sh, r) {
            missed = true;
            error!("[error] {r} not found");
        }
    }

    if missed {
        bail!("some required executables is not found")
    }

    Ok(())
}

pub fn exe_in_path(sh: &Shell, exe: &str) -> bool {
    cmd!(sh, "which {exe}")
        .quiet()
        .ignore_stdout()
        .run()
        .is_ok()
}

pub fn create_and_cd_to_artifacts_dir(sh: &Shell) -> Result<()> {
    std::fs::create_dir_all(ARTIFACTS_DIR).context("failed to create artifacts dir")?;
    sh.change_dir(ARTIFACTS_DIR);
    std::env::set_current_dir(ARTIFACTS_DIR).context("failed to change current dir")?;
    Ok(())
}

pub fn create_dir(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        debug!("creating directory {}", path.display());
        std::fs::create_dir_all(path)
            .with_context(|| format!("failed to create {}", path.display()))?;
    }
    Ok(())
}

pub fn save_config(dir: impl AsRef<Path>, file: &str, text: &str) -> Result<()> {
    let dir = dir.as_ref();
    let path = dir.join(file);
    debug!("writing {}", path.display());
    std::fs::write(path, text)
        .with_context(|| format!("failed to save {file} to {}", dir.display()))?;
    Ok(())
}

pub fn save_json_config<T: Serialize>(dir: impl AsRef<Path>, file: &str, data: &T) -> Result<()> {
    let dir = dir.as_ref();
    let text = serde_json::to_string_pretty(data)
        .with_context(|| format!("failed to serialize {}", dir.join(file).display()))?;
    save_config(dir, file, &text)
}

pub fn path_to_str(p: impl AsRef<Path>) -> Result<String> {
    let p = p.as_ref();
    p.to_str()
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("path {} is not valid utf-8", p.display()))
}
