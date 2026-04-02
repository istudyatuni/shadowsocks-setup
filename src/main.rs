use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use xshell::Shell;

use args::{Args, ShadowsocksArgs, XrayArgs};

mod args;
mod cipher;
mod github;
mod install;
mod version;

#[cfg(target_env = "musl")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let args = Args::parse();
    let sh = Shell::new()?;

    // disable in dev build
    if cfg!(not(debug_assertions)) {
        if args.prefer_root() {
            // ask to continue when running under sudo or normal user
            if is_sudo_not_root() || sudo::check() != sudo::RunningAs::Root {
                let cont = inquire::Confirm::new(
                    "It's prefered to run this as root user (not with sudo). Continue anyway?",
                )
                .with_default(false)
                .prompt()?;
                if !cont {
                    return Ok(());
                }
            }
        }

        // escalate if need root
        if args.need_root() && sudo::check() != sudo::RunningAs::Root {
            eprintln!("escalating to root");
            if sudo::escalate_if_needed().map_err(|e| anyhow!("{e}"))? != sudo::RunningAs::Root {
                bail!("This script requires root privileges");
            }
        }
    }

    init_logger()?;

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

fn init_logger() -> Result<()> {
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .without_time()
            .with_file(true)
            .with_line_number(true)
            .finish(),
    )
    .context("failed to init logging")?;
    Ok(())
}

fn is_sudo_not_root() -> bool {
    std::env::var_os("SUDO_USER").is_some()
}
