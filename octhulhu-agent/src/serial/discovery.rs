use std::fmt::{Display, Formatter};
use cthulhu_config::octhulhu::OcthulhuConfig;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_serial::SerialPortType;
use tracing::info;

#[derive(Debug, Clone)]
pub enum DiscoveredDeviceLocation {
    LocalTTY(PathBuf),
    RemoteTCP(SocketAddr),
}

impl Display for DiscoveredDeviceLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveredDeviceLocation::LocalTTY(s) => write!(f, "{}", s.display()),
            DiscoveredDeviceLocation::RemoteTCP(a) => write!(f, "{a}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub serial_number: String,
    pub location: DiscoveredDeviceLocation,
}
pub async fn discover_devices(
    config: &OcthulhuConfig,
) -> color_eyre::Result<Vec<DiscoveredDevice>> {
    info!("Discovering devices...");
    let mut devices: Vec<DiscoveredDevice> = Vec::new();

    // Discover USB devices.
    for port in tokio_serial::available_ports()? {
        let name = port.port_name;
        if let SerialPortType::UsbPort(pd) = port.port_type {
            if ((pd.vid == 0x16c0 && pd.pid == 0x27dd) || (pd.vid == 0x05a6 && pd.pid == 0x0009))
                && pd.manufacturer.unwrap_or("".to_string()) == "Cthulhu"
                && pd.product.unwrap_or("".to_string()) == "Octhulhu"
                && let Some(usn) = pd.serial_number
            {
                devices.push(DiscoveredDevice {
                    serial_number: usn.clone(),
                    location: DiscoveredDeviceLocation::LocalTTY(name.parse()?),
                });
            }
        }
    }

    // Discover TCP devices
    for port in config.network_serials.iter() {
        let a: IpAddr = port.host.as_str().parse()?;
        let a: SocketAddr = SocketAddr::new(a, port.port);
        info!("Probing {a}...");
        let p = TcpStream::connect(a).await?;
        let (r, mut w) = p.into_split();
        let mut r = BufReader::new(r);
        w.write_all("\n\nI\n".as_bytes()).await?;
        let sn_opt = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let mut buffer = String::new();
                r.read_line(&mut buffer).await?;
                let line = buffer.trim();
                if line.starts_with("I")
                    && let Some(sn) = line.strip_prefix("I")
                {
                    return color_eyre::eyre::Ok(sn.to_string());
                }
            }
        })
        .await.ok();

        if let Some(sn_r) = sn_opt {
            let sn = sn_r?;
            devices.push(DiscoveredDevice {
                serial_number: sn,
                location: DiscoveredDeviceLocation::RemoteTCP(a),
            });
        }
    }

    Ok(devices)
}
