//! AT Command terminal interface

use crate::error::AtError;
use crate::serial::{SerialConfig, SerialConnection};
use log::{debug, info};
use std::io::{self, Write};
use std::str;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Represents received data from the serial port
#[derive(Debug, Clone)]
pub enum ReceivedData {
    /// UTF-8 text data
    Text(String),
    /// Binary data
    Binary(Vec<u8>),
}

/// Trait for handling received data
pub trait DataHandler: Send {
    /// Called when data is received from the serial port
    fn handle_data(&mut self, data: ReceivedData);
}

/// Simple implementation that prints received data to stdout
pub struct SimpleDataHandler;

impl DataHandler for SimpleDataHandler {
    fn handle_data(&mut self, data: ReceivedData) {
        match data {
            ReceivedData::Text(text) => {
                print!("{}", text);
                let _ = io::stdout().flush();
            }
            ReceivedData::Binary(bytes) => {
                println!("\nReceived binary data: {:02X?}", bytes);
            }
        }
    }
}

/// AT command terminal to interact with serial devices
pub struct AtCommandTerminal {
    connection: SerialConnection,
    reader_thread: Option<thread::JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl AtCommandTerminal {
    /// Create a new AT command terminal with the specified configuration
    pub fn new(config: &SerialConfig) -> Result<Self, AtError> {
        let connection = SerialConnection::open(config)?;
        let running = Arc::new(Mutex::new(true));

        Ok(Self {
            connection,
            reader_thread: None,
            running,
        })
    }

    /// Start the reader thread with a custom data handler
    pub fn start_reader<H: DataHandler + 'static>(&mut self, handler: H) -> Result<(), AtError> {
        if self.reader_thread.is_some() {
            return Err(AtError::Other("Reader thread already started".to_string()));
        }

        let mut reader_port = self.connection.try_clone()?;
        let running = Arc::clone(&self.running);
        let handler = Arc::new(Mutex::new(handler));

        self.reader_thread = Some(thread::spawn(move || {
            let mut buffer = [0u8; 1024];

            while *running.lock().unwrap() {
                match reader_port.read(&mut buffer) {
                    Ok(count) if count > 0 => {
                        let data = match str::from_utf8(&buffer[..count]) {
                            Ok(s) => ReceivedData::Text(s.to_string()),
                            Err(_) => ReceivedData::Binary(buffer[..count].to_vec()),
                        };

                        if let Ok(mut handler) = handler.lock() {
                            handler.handle_data(data);
                        }
                    }
                    Ok(_) => thread::sleep(Duration::from_millis(5)),
                    Err(_) => thread::sleep(Duration::from_millis(5)),
                }
            }
        }));

        Ok(())
    }

    /// Send an AT command
    pub fn send_command(&mut self, command: &str) -> Result<(), AtError> {
        debug!("Sending command: {}", command.trim());

        // Format command to ensure it ends with \r\n
        let formatted_cmd = if command.ends_with("\r\n") {
            command.to_string()
        } else if command.ends_with("\r") || command.ends_with("\n") {
            format!("{}\r\n", command.trim_end())
        } else {
            format!("{}\r\n", command)
        };

        self.connection.write(formatted_cmd.as_bytes())?;
        Ok(())
    }

    /// Get the list of available serial ports
    pub fn list_available_ports() -> Result<Vec<String>, AtError> {
        let ports = SerialConnection::list_ports()?;
        Ok(ports.into_iter().map(|info| info.port_name).collect())
    }

    /// Stop the terminal and its reader thread
    pub fn stop(&mut self) {
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }

        if let Some(thread) = self.reader_thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for AtCommandTerminal {
    fn drop(&mut self) {
        self.stop();
    }
}