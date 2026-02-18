use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use xshell::{Shell, cmd};

pub mod input;
mod network;
pub mod shadowsocks;
pub mod xray;
pub mod xray_config;

const ARTIFACTS_DIR: &str = "artifacts";

pub fn check_requirements(sh: &Shell, bin_reqs: &[&str]) -> Result<()> {
    println!("[prepare] checking required executables");
    let mut missed = false;
    for r in bin_reqs {
        if !exe_in_path(sh, r) {
            missed = true;
            eprintln!("[error] {r} not found");
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

pub fn path_to_str(p: &Path) -> Result<String> {
    p.to_str()
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("path {} is not valid utf-8", p.display()))
}
