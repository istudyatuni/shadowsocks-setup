use anyhow::{Result, bail};
use xshell::{Shell, cmd};

pub mod input;
mod network;
pub mod shadowsocks;
pub mod xray;

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
