use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[clap(long, short)]
    pub config: PathBuf,
}
