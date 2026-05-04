use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "kramer")]
pub struct KramerCli {
    #[command(subcommand)]
    pub command: Option<KramerCommand>,
}

#[derive(Debug, clap::Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum KramerCommand {
    LogLocation,
    Log,
}
