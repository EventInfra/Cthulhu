use crate::serial::discovery::{DiscoveredDevice, DiscoveredDeviceLocation};
use color_eyre::eyre::{OptionExt, eyre};
use serial2_tokio::SerialPort;
use std::collections::BTreeMap;
use std::io::Write;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{Mutex, broadcast};
use tracing::warn;

pub mod discovery;

#[derive(Clone)]
pub enum SerialPortMessage {
    UnknownResponse {
        serial_number: String,
        line: String,
    },
    ModuleUpdate {
        serial_number: String,
        present: [bool; 8],
    },
    PresenceUpdate {
        serial_number: String,
        present: [bool; 8],
    },
}

#[derive(Clone)]
pub struct SerialPortManager {
    broadcast_channel: Sender<SerialPortMessage>,
    inner: Arc<Mutex<BTreeMap<String, SerialPortInner>>>,
}

pub type BoxedAsyncRead = Box<dyn AsyncRead + Send + Unpin>;
pub type BoxedAsyncWrite = Box<dyn AsyncWrite + Send + Unpin>;

impl SerialPortManager {
    pub fn new() -> Self {
        Self {
            broadcast_channel: broadcast::channel(1000).0,
            inner: Arc::new(Mutex::new(Default::default())),
        }
    }

    pub fn receiver(&self) -> Receiver<SerialPortMessage> {
        self.broadcast_channel.subscribe()
    }

    async fn register_port(
        &self,
        serial_number: &str,
        writer: BoxedAsyncWrite,
    ) -> color_eyre::Result<()> {
        let mut handle = self.inner.lock().await;
        if !handle.contains_key(serial_number) {
            handle.insert(serial_number.to_string(), SerialPortInner { writer });
        }
        Ok(())
    }

    async fn write_all_to_port(&self, serial_number: &str, data: &[u8]) -> color_eyre::Result<()> {
        let mut handle = self.inner.lock().await;
        let writer = &mut handle
            .get_mut(serial_number)
            .ok_or_eyre("unknown serial")?
            .writer;
        writer.write_all(data).await?;
        Ok(())
    }

    pub async fn request_presence_update(&self, serial_number: &str) -> color_eyre::Result<()> {
        let mut buf = Vec::new();
        write!(buf, "P\r\n")?;
        self.write_all_to_port(&serial_number, &mut buf).await
    }

    pub async fn request_module_update(&self, serial_number: &str) -> color_eyre::Result<()> {
        let mut buf = Vec::new();
        write!(buf, "M\r\n")?;
        self.write_all_to_port(&serial_number, &mut buf).await
    }

    pub async fn request_all_presence_updates(&self) -> color_eyre::Result<()> {
        let serial_numbers = self.inner.lock().await.keys().cloned().collect::<Vec<_>>();
        for sn in serial_numbers {
            self.request_presence_update(&sn).await?;
        }
        Ok(())
    }

    pub async fn request_all_module_updates(&self) -> color_eyre::Result<()> {
        let serial_numbers = self.inner.lock().await.keys().cloned().collect::<Vec<_>>();
        for sn in serial_numbers {
            self.request_module_update(&sn).await?;
        }
        Ok(())
    }

    pub async fn set_led_color(
        &self,
        serial_number: &str,
        idx: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> color_eyre::Result<()> {
        let mut buf = Vec::new();
        write!(buf, "S{:02X}{:02X}{:02X}{:02X}\r\nF\r\n", idx, r, g, b)?;
        self.write_all_to_port(&serial_number, &mut buf).await
    }
}

struct SerialPortInner {
    writer: BoxedAsyncWrite,
}

impl DiscoveredDeviceLocation {
    async fn connect(&self) -> color_eyre::Result<(BoxedAsyncRead, BoxedAsyncWrite)> {
        match self {
            DiscoveredDeviceLocation::LocalTTY(t) => {
                let port = SerialPort::open(t, 115200)?;
                let port2 = port.try_clone()?;
                Ok((Box::new(port), Box::new(port2)))
            }
            DiscoveredDeviceLocation::RemoteTCP(a) => {
                let s = TcpStream::connect(a).await?;
                let (r, w) = s.into_split();
                Ok((Box::new(r), Box::new(w)))
            }
        }
    }
}

impl DiscoveredDevice {
    pub async fn connect_to(&self, manager: &SerialPortManager) -> color_eyre::Result<()> {
        let (r, mut w) = self.location.connect().await?;

        // Send some enters to ensure the state is good.
        w.write_all("\r\n\r\n\r\n".as_bytes()).await?;

        manager.register_port(&self.serial_number, w).await?;

        let sender = manager.broadcast_channel.clone();
        let serial = self.serial_number.clone();

        // Start worker task.
        tokio::task::spawn(process_device_messages(sender, serial, r));

        Ok(())
    }
}

async fn process_device_messages(
    sender: Sender<SerialPortMessage>,
    serial_number: String,
    reader: BoxedAsyncRead,
) {
    let mut r = BufReader::new(reader);
    loop {
        let mut buffer = String::new();
        r.read_line(&mut buffer)
            .await
            .expect("error polling device");
        let line = buffer.trim();
        if let Err(e) = process_single_line(sender.clone(), serial_number.clone(), line).await {
            warn!("Error processing line from device {serial_number}: {e}");
        }
    }
}

async fn process_single_line(
    sender: Sender<SerialPortMessage>,
    serial_number: String,
    line: &str,
) -> color_eyre::Result<()> {
    let mut chars = line.chars();

    let cmdr = chars.next();

    match cmdr.ok_or_eyre("no command in line")? {
        'P' => {
            let d: Vec<bool> = chars.take(8).map(|c| c == '1').collect();
            if d.len() == 8 {
                let present = d
                    .try_into()
                    .map_err(|_| eyre!("invalid presence response"))?;

                let _ = sender.send(SerialPortMessage::PresenceUpdate {
                    serial_number,
                    present,
                });
            }
        }
        'M' => {
            let d: Vec<bool> = chars.take(8).map(|c| c == '1').collect();
            if d.len() == 8 {
                let present = d.try_into().map_err(|_| eyre!("invalid module response"))?;

                let _ = sender.send(SerialPortMessage::ModuleUpdate {
                    serial_number,
                    present,
                });
            }
        }
        _ => {
            let _ = sender.send(SerialPortMessage::UnknownResponse {
                serial_number,
                line: line.to_string(),
            });
        }
    }

    Ok(())
}
