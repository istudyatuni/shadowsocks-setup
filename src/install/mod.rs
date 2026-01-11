use anyhow::{bail, Result};
use xshell::{cmd, Shell};

pub mod input;
mod network;
pub mod shadowsocks;

pub fn check_requirements(sh: &Shell, bin_reqs: &[&str]) -> Result<()> {
    println!("[prepare] checking required executables");
    let mut missed = false;
    for r in bin_reqs {
        if cmd!(sh, "which {r}").quiet().ignore_stdout().run().is_err() {
            missed = true;
            eprintln!("[error] {r} not found");
        }
    }

    if missed {
        bail!("some required executables is not found")
    }

    Ok(())
}
