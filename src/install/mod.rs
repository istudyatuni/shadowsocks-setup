use anyhow::{Context, Result, bail};
use xshell::{Shell, cmd};

pub mod input;
mod network;
pub mod shadowsocks;
pub mod xray;

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
