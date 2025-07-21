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
    dot_duration: Duration,
    dash_duration: Duration,
    gap_duration: Duration,
    word_gap_duration: Duration,
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

        // Standard Morse timing (adjust as needed)
        let dot_duration = Duration::from_millis(200);
        let dash_duration = Duration::from_millis(600); // 3x dot duration
        let gap_duration = Duration::from_millis(200); // Between elements
        let word_gap_duration = Duration::from_millis(1400); // 7x dot duration

        Ok(Self {
            port,
            dot_duration,
            dash_duration,
            gap_duration,
            word_gap_duration,
        })
    }

    pub fn send_morse(&mut self, morse_code: &str) -> Result<(), SerialError> {
        for (i, token) in morse_code.split_whitespace().enumerate() {
            if i > 0 {
                // Add gap between morse characters
                std::thread::sleep(self.gap_duration);
            }

            if token == "/" {
                // Word separator
                std::thread::sleep(self.word_gap_duration);
                continue;
            }

            for (j, symbol) in token.chars().enumerate() {
                if j > 0 {
                    // Small gap between dots and dashes within a character
                    std::thread::sleep(Duration::from_millis(50));
                }

                match symbol {
                    '.' => {
                        self.send_dot()?;
                    }
                    '-' => {
                        self.send_dash()?;
                    }
                    _ => {
                        // Ignore unknown symbols
                        eprintln!("Warning: Unknown Morse symbol {symbol}");
                    }
                }
            }
        }

        Ok(())
    }

    fn send_dot(&mut self) -> Result<(), SerialError> {
        self.port.write_all(b"1")?; // Signal ON
        self.port.flush()?;
        std::thread::sleep(self.dot_duration);

        self.port.write_all(b"0")?; // Signal OFF
        self.port.flush()?;

        Ok(())
    }

    fn send_dash(&mut self) -> Result<(), SerialError> {
        self.port.write_all(b"1")?; // Signal ON
        self.port.flush()?;
        std::thread::sleep(self.dash_duration);

        self.port.write_all(b"0")?; // Signal OFF
        self.port.flush()?;

        Ok(())
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

    pub fn set_timing(&mut self, dot_ms: u64, dash_multiplier: u64) {
        self.dot_duration = Duration::from_millis(dot_ms);
        self.dash_duration = Duration::from_millis(dot_ms * dash_multiplier);
        self.gap_duration = Duration::from_millis(dot_ms);
        self.word_gap_duration = Duration::from_millis(dot_ms * 7);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_ports() {
        // This test will only work on systems with serial ports
        match SerialSender::list_ports() {
            Ok(ports) => {
                println!("Available ports: {:?}", ports);
                // Test passes if we can list ports without error
            }
            Err(e) => {
                println!("No ports available or error: {e}");
                // This is okay for testing environments
            }
        }
    }
}
