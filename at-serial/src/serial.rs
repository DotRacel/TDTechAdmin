//! Serial port connection handling.

use crate::error::AtError;
use log::{debug, error, warn};
use serialport::SerialPort;
use std::io::{self, Read, Write};
use std::time::Duration;

/// Configuration for a serial connection.
#[derive(Debug, Clone)]
pub struct SerialConfig {
    /// Path to the serial device.
    pub path: String,
    /// Baud rate.
    pub baud_rate: u32,
    /// Timeout for read/write operations.
    pub timeout: Duration,
    /// Flow control type.
    pub flow_control: serialport::FlowControl,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            path: "COM1".to_string(),
            baud_rate: 115200,
            timeout: Duration::from_millis(10),
            flow_control: serialport::FlowControl::None,
        }
    }
}

/// A connection to a serial device.
pub struct SerialConnection {
    port: Box<dyn SerialPort>,
}

impl SerialConnection {
    /// Opens a new serial connection with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use at_serial::{SerialConfig, SerialConnection};
    /// use std::time::Duration;
    ///
    /// let config = SerialConfig {
    ///     path: "/dev/ttyUSB0".to_string(),
    ///     baud_rate: 9600,
    ///     ..Default::default()
    /// };
    ///
    /// let conn = SerialConnection::open(&config).expect("Failed to open serial port");
    /// ```
    pub fn open(config: &SerialConfig) -> Result<Self, AtError> {
        debug!("Opening serial port {} at {} baud", config.path, config.baud_rate);

        let port = serialport::new(&config.path, config.baud_rate)
            .timeout(config.timeout)
            .flow_control(config.flow_control)
            .open()?;

        Ok(Self { port })
    }

    /// Lists all available serial ports.
    pub fn list_ports() -> Result<Vec<serialport::SerialPortInfo>, AtError> {
        serialport::available_ports().map_err(|e| AtError::Other(e.to_string()))
    }

    /// Writes data to the serial port.
    ///
    /// This method handles platform-specific issues and automatically flushes.
    pub fn write(&mut self, data: &[u8]) -> Result<usize, AtError> {
        match self.port.write_all(data) {
            Ok(_) => {
                // On Windows, flush can cause "Incorrect function" errors, so we handle any errors quietly
                let _ = self.port.flush();
                Ok(data.len())
            }
            Err(e) => {
                error!("Failed to write data: {}", e);
                Err(AtError::IoError(e))
            }
        }
    }

    /// Reads data from the serial port.
    ///
    /// Returns the number of bytes read.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, AtError> {
        match self.port.read(buffer) {
            Ok(count) => Ok(count),
            Err(e) if e.kind() == io::ErrorKind::TimedOut => Ok(0), // Timeout is not an error
            Err(e) => {
                warn!("Serial port read error: {}", e);
                Err(AtError::IoError(e))
            }
        }
    }

    /// Creates a clone of the underlying port.
    ///
    /// Useful for creating reader/writer pairs.
    pub fn try_clone(&self) -> Result<Self, AtError> {
        let port = self.port.try_clone()?;
        Ok(Self { port })
    }
}