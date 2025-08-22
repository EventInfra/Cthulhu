use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[clap(long, short)]
    pub config: PathBuf,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    ListBoards,
    Daemon,
}
