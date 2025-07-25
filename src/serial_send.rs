use serialport::{SerialPort, SerialPortType};
use std::io::Write;
use std::time::Duration;

#[derive(Debug)]
pub enum SerialError {
    PortError(serialport::Error),
    IoError(std::io::Error),
    InvalidPort(String),
}

impl std::fmt::Display for SerialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerialError::PortError(e) => write!(f, "Serial port error: {e}"),
            SerialError::IoError(e) => write!(f, "IO error: {e}"),
            SerialError::InvalidPort(port) => write!(f, "Invalid serial port: {port}"),
        }
    }
}

impl std::error::Error for SerialError {}

impl From<serialport::Error> for SerialError {
    fn from(err: serialport::Error) -> Self {
        SerialError::PortError(err)
    }
}

impl From<std::io::Error> for SerialError {
    fn from(err: std::io::Error) -> Self {
        SerialError::IoError(err)
    }
}

pub struct SerialSender {
    port: Box<dyn SerialPort>,
}

impl SerialSender {
    pub fn new(port_path: &str, baud_rate: u32) -> Result<Self, SerialError> {
        // Validate port exists
        if !std::path::Path::new(port_path).exists() {
            return Err(SerialError::InvalidPort(port_path.to_string()));
        }

        let port = serialport::new(port_path, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open()?;

        Ok(Self { port })
    }

    pub fn list_ports() -> Result<Vec<String>, SerialError> {
        let ports = serialport::available_ports()?;
        let port_names: Vec<String> = ports
            .iter()
            .map(|port| match &port.port_type {
                SerialPortType::UsbPort(info) => {
                    format!(
                        "{} (USB: {})",
                        port.port_name,
                        info.product.as_deref().unwrap_or("Unknown")
                    )
                }
                SerialPortType::BluetoothPort => {
                    format!("{} (Bluetooth)", port.port_name)
                }
                SerialPortType::PciPort => {
                    format!("{} (PCI)", port.port_name)
                }
                SerialPortType::Unknown => {
                    format!("{} (Unknown)", port.port_name)
                }
            })
            .collect();

        Ok(port_names)
    }

    pub fn send_raw(&mut self, data: &[u8]) -> Result<(), SerialError> {
        self.port.write_all(data)?;
        self.port.flush()?;
        Ok(())
    }
}
