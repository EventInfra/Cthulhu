use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    ListBoards,
    Daemon(DaemonArgs),
}

#[derive(Args, Debug, Clone)]
pub struct DaemonArgs {
    #[clap(long, short)]
    pub config: PathBuf,
}