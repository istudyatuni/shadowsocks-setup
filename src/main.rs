use anyhow::{Result, anyhow, bail};
use clap::Parser;
use xshell::Shell;

use args::{Args, ShadowsocksArgs, XrayArgs};

mod args;
mod cipher;
mod github;
mod install;
mod version;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let args = Args::parse();
    let sh = Shell::new()?;

    // disable in dev build
    if cfg!(not(debug_assertions)) && args.need_root() && sudo::check() != sudo::RunningAs::Root {
        eprintln!("escalating to root");
        if sudo::escalate_if_needed().map_err(|e| anyhow!("{e}"))? != sudo::RunningAs::Root {
            bail!("This script requires sudo");
        }
    }

    match args {
        Args::Shadowsocks { cmd } => match cmd {
            ShadowsocksArgs::Install(args) => install::shadowsocks::install(&sh, args)?,
            ShadowsocksArgs::Update(args) => install::shadowsocks::update(&sh, args)?,
            ShadowsocksArgs::Uninstall => install::shadowsocks::uninstall(&sh)?,
        },
        Args::Xray { cmd } => match cmd {
            XrayArgs::Install(args) => install::xray::run_install_manager(&sh, args)?,
            XrayArgs::InstallStep { step } => install::xray::install(&sh, step)?,
        },
    }

    Ok(())
}
