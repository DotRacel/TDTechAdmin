use at_serial::{AtCommandTerminal, SimpleDataHandler, SerialConfig};
use std::env;
use std::io::{self, Write};

const DEFAULT_TTY: &str = "COM8";
const DEFAULT_BAUD: u32 = 115200;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Get the TTY path and baud rate from command line args or use defaults
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_TTY.to_string());

    let baud = env::args()
        .nth(2)
        .map(|arg| arg.parse::<u32>().unwrap_or(DEFAULT_BAUD))
        .unwrap_or(DEFAULT_BAUD);

    // Display available ports
    println!("Available serial ports:");
    for port in AtCommandTerminal::list_available_ports()? {
        println!("  {}", port);
    }
    println!("Opening serial port {} at {} baud", path, baud);

    // Create configuration
    let config = SerialConfig {
        path,
        baud_rate: baud,
        ..Default::default()
    };

    // Create terminal and start reader thread
    let mut terminal = AtCommandTerminal::new(&config)?;
    terminal.start_reader(SimpleDataHandler)?;

    println!("Type AT commands and press Enter to send. Press Ctrl+C to quit.");

    // Main command loop
    let mut input = String::new();
    loop {
        // Clear input buffer
        input.clear();

        // Prompt and get input
        print!("> ");
        io::stdout().flush()?;

        // Read a line from stdin
        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        // Trim the input (remove trailing newline)
        let command = input.trim();

        if command.is_empty() {
            continue;
        }

        // Echo the AT command
        println!("Sending: {}", command);

        // Send the command
        terminal.send_command(command)?;
    }

    Ok(())
}