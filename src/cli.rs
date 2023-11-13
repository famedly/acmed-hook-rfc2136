use clap::{Parser, Subcommand};
use hickory_client::rr::Name;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, long_about = None)]
pub(crate) struct CliArgs {
    #[arg(short, long)]
    pub config: PathBuf,

    #[arg(short, long)]
    pub identifier: Name,

    #[arg(short, long)]
    pub proof: String,

    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Subcommand)]
pub enum Commands {
    /// sets the challenge record
    Set,
    /// removes the challenge record
    Unset,
}

pub(crate) fn parse() -> CliArgs {
    CliArgs::parse()
}
