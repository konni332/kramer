use clap::Parser;
use kramer_core::{log_location, run_engine};

use crate::cli::{KramerCli, KramerCommand};

mod cli;

fn main() -> miette::Result<()> {
    let cli = KramerCli::parse();
    if let Some(cmd) = cli.command {
        execute_cli_command(cmd)?;
    } else {
        run_engine();
    }

    Ok(())
}

fn execute_cli_command(cmd: KramerCommand) -> miette::Result<()> {
    match cmd {
        KramerCommand::Log => open_log_file()?,
        KramerCommand::LogLocation => {
            println!("kramer log file at: {}", log_location().display());
        }
    }

    Ok(())
}

fn open_log_file() -> miette::Result<()> {
    let path = log_location();

    std::fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| miette::miette!("failed to create log dir: {e}"))?;
    std::fs::File::create(&path).map_err(|e| miette::miette!("failed to create log file: {e}"))?;
    open::that(&path).map_err(|e| miette::miette!("failed to open log file: {e}"))?;

    Ok(())
}
