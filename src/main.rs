use std::fs::create_dir_all;

use anyhow::{Context, Result, bail};
use clap::Parser;
use xshell::Shell;

use args::Args;

mod args;
mod cipher;
mod github;
mod install;
mod version;

const ARTIFACTS_DIR: &str = "shadowsocks-artifacts";

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let args = Args::parse();
    let sh = Shell::new()?;

    // disable in dev build
    if cfg!(not(debug_assertions)) && sudo::check() != sudo::RunningAs::Root {
        bail!("This script requires sudo");
    }

    create_dir_all(ARTIFACTS_DIR).context("failed to create artifacts dir")?;
    sh.change_dir(ARTIFACTS_DIR);
    std::env::set_current_dir(ARTIFACTS_DIR).context("failed to change current dir")?;

    match args {
        Args::Install(args) => install::shadowsocks::install(&sh, args)?,
        Args::Update(args) => install::shadowsocks::update(&sh, args)?,
        Args::Uninstall => install::shadowsocks::uninstall(&sh)?,
    }

    Ok(())
}
