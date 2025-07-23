use std::io::Write;
use color_eyre::eyre::eyre;
use serial2_tokio::SerialPort;
use tokio_serial::SerialPortType;
use tokio::io::AsyncWriteExt;

pub async fn connect_serial_by_sn(sn: &str) -> color_eyre::Result<SerialPort> {
    let ports = tokio_serial::available_ports()?;
    for port in ports {
        let name = port.port_name;
        if let SerialPortType::UsbPort(pd) = port.port_type {
            if pd.vid == 0x16c0
                && pd.pid == 0x27dd
                && pd.manufacturer.unwrap_or("".to_string()) == "Cthulhu"
                && pd.product.unwrap_or("".to_string()) == "Octhulhu"
                && let Some(usn) = pd.serial_number
                && usn == sn
            {
                return Ok(SerialPort::open(name, 115200)?);
            }
        }
    }
    Err(eyre!("No serial port with serial number {sn} found."))
}

pub async fn set_led_color(port: &mut SerialPort, idx: u8, r: u8, g: u8, b: u8) -> color_eyre::Result<()> {
    let mut buf = Vec::new();
    write!(buf, "S{:02X}{:02X}{:02X}{:02X}\r\nF\r\n", idx, r, g, b)?;
    port.write_all(&buf).await?;
    Ok(())
}

