use crate::args::{Cli, Commands};
use clap::Parser;
use cthulhu_config::octhulhu::OcthulhuConfig;

mod args;
mod daemon;

mod serial;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let args: Cli = Cli::parse();

    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();

    let config = OcthulhuConfig::from_file(&args.config).await?;

    match args.command {
        Commands::ListBoards => {
            let devices = serial::discovery::discover_devices(&config).await?;
            for device in devices {
                println!("Device at {}: {}", device.location, device.serial_number);
            }

        }
        Commands::Daemon => {
            daemon::daemon(config).await?;
        }
    }

    Ok(())
}
