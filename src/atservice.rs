use serialport::{SerialPort};
use std::time::Duration;
use std::io::{Read, Write};

const DEFAULT_TIMEOUT_MS: u64 = 1000;

pub struct ATService {
    port: Box<dyn SerialPort>,
}

impl ATService {
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let port_name = port_name;
        let baud_rate = baud_rate;
        
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .open()?;
            
        Ok(Self { port })
    }

    pub fn send_command(&mut self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.port.write_all(format!("{}\r\n", command).as_bytes())?;
        self.read_response()
    }

    pub fn read_response(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut response = String::new();
        let mut line = String::new();

        loop {
            line.clear();
            while !line.ends_with('\n') {
                let mut buffer = [0; 1];
                match self.port.read(&mut buffer) {
                    Ok(_) => line.push(buffer[0] as char),
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        return Err("Timeout waiting for response".into());
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            response.push_str(&line);
            if line.trim() == "OK" || line.trim() == "ERROR" {
                break;
            }
        }
        Ok(response)
    }
}
