use std::fs::create_dir_all;

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use xshell::Shell;

use args::{Args, ShadowsocksArgs, XrayArgs};

mod args;
mod cipher;
mod github;
mod install;
mod version;

const ARTIFACTS_DIR: &str = "artifacts";

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let args = Args::parse();
    let sh = Shell::new()?;

    // disable in dev build
    if cfg!(not(debug_assertions))
        && !args.need_root()
        && sudo::escalate_if_needed().map_err(|e| anyhow!("{e}"))? != sudo::RunningAs::Root
    {
        bail!("This script requires sudo");
    }

    create_dir_all(ARTIFACTS_DIR).context("failed to create artifacts dir")?;
    sh.change_dir(ARTIFACTS_DIR);
    std::env::set_current_dir(ARTIFACTS_DIR).context("failed to change current dir")?;

    match args {
        Args::Shadowsocks { cmd } => match cmd {
            ShadowsocksArgs::Install(args) => install::shadowsocks::install(&sh, args)?,
            ShadowsocksArgs::Update(args) => install::shadowsocks::update(&sh, args)?,
            ShadowsocksArgs::Uninstall => install::shadowsocks::uninstall(&sh)?,
        },
        Args::Xray { cmd } => match cmd {
            XrayArgs::Install(args) => install::xray::run_install_manager(&sh, args)?,
        },
    }

    Ok(())
}
