use crate::args::{Cli, Commands};
use clap::Parser;
use tokio_serial::SerialPortType;
use cthulhu_config::octhulhu::OcthulhuConfig;
use crate::daemon::daemon;

mod args;
mod daemon;

mod serial;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let args: Cli = Cli::parse();

    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();

    match args.command {
        Commands::ListBoards => {
            let ports = tokio_serial::available_ports()?;
            for port in ports {
                let name = port.port_name;
                if let SerialPortType::UsbPort(pd) = port.port_type {
                    if pd.vid == 0x16c0
                        && pd.pid == 0x27dd
                        && pd.manufacturer.unwrap_or("".to_string()) == "Cthulhu"
                        && pd.product.unwrap_or("".to_string()) == "Octhulhu"
                        && let Some(sn) = pd.serial_number
                    {
                        println!("Device at {}: {}", name, sn);
                    }
                }
            }
        }
        Commands::Daemon(c) => {
            let config = OcthulhuConfig::from_file(&c.config).await?;
            daemon(config).await?;
        }
    }

    Ok(())
}
