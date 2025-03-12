use std::fmt::format;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{self, Color},
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use at_serial::{AtCommandTerminal, DataHandler, ReceivedData, SerialConfig};
use std::sync::mpsc::{self, Receiver, Sender};

pub struct Terminal {
    input_buffer: String,
    command_history: Vec<String>,
    output_history: Vec<(String, Color)>,
    at_terminal: AtCommandTerminal,
    data_receiver: Receiver<ReceivedData>,
}

// This owns the sender and implements DataHandler
struct ChannelHandler {
    sender: Sender<ReceivedData>,
}

impl DataHandler for ChannelHandler {
    fn handle_data(&mut self, data: ReceivedData) {
        let _ = self.sender.send(data);
    }
}

impl Terminal {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        let mut terminal = Self {
            input_buffer: String::new(),
            command_history: Vec::new(),
            output_history: Vec::new(),
            at_terminal: AtCommandTerminal::new(
                &SerialConfig {
                    path: String::from("COM8"),
                    baud_rate: 115200,
                    ..Default::default()
                }
            ).unwrap(),
            data_receiver: receiver,
        };

        let handler = ChannelHandler { sender };
        terminal.at_terminal.start_reader(handler).unwrap();

        terminal
    }

    fn display_output(&mut self, message: &str, color: Color) {
        self.output_history.push((message.to_string(), color));
        execute!(
            io::stdout(),
            style::SetForegroundColor(color),
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveToColumn(0),
        )
            .unwrap();
        println!("{}", message);
        execute!(io::stdout(), style::ResetColor).unwrap();
    }

    fn process_command(&mut self, command: &str) -> bool {
        match command.trim() {
            "exit" | "quit" => return false,
            "clear" => {
                execute!(
                    io::stdout(),
                    terminal::Clear(ClearType::All),
                    cursor::MoveTo(0, 0)
                )
                    .unwrap();
                self.output_history.clear();
            }
            "help" => {
                self.display_output("available commands:", Color::White);
                self.display_output("  help", Color::White);
                self.display_output("  clear", Color::White);
                self.display_output("  exit", Color::White);
            }
            "" => {}
            _ => {
                self.at_terminal.send_command(command).unwrap();
            }
        }
        true
    }

    // Count how many lines a string will take when displayed
    fn count_display_lines(&self, text: &str) -> u16 {
        // Get terminal width to calculate wrapping
        let (width, _) = terminal::size().unwrap_or((80, 24));

        let mut line_count = 0;
        for line in text.lines() {
            // Empty lines still count as 1 line
            if line.is_empty() {
                line_count += 1;
                continue;
            }

            // Calculate how many terminal lines this logical line will take
            let chars_count = line.chars().count() as u16;
            let wrapped_lines = (chars_count + width - 1) / width; // Ceiling division
            line_count += std::cmp::max(1, wrapped_lines); // At least 1 line
        }

        // If text is empty, it still takes 1 line
        std::cmp::max(1, line_count)
    }

    fn redraw(&self) -> io::Result<()> {
        execute!(io::stdout(), terminal::Clear(ClearType::All))?;

        // Track current line position
        let mut current_line = 0;

        // show history
        for (message, color) in &self.output_history {
            execute!(
                io::stdout(), 
                cursor::MoveTo(0, current_line),
                style::SetForegroundColor(*color)
            )?;

            // Print the message line by line to handle newlines properly
            for line in message.lines() {
                execute!(io::stdout(), cursor::MoveTo(0, current_line))?;
                print!("{}", line);
                current_line += 1;
            }

            // If message doesn't end with newline, we still need to move to next line
            if !message.ends_with('\n') {
                current_line += 1;
            }
        }

        // show prompt
        execute!(
            io::stdout(),
            cursor::MoveTo(0, current_line),
            style::SetForegroundColor(Color::Cyan)
        )?;
        print!("> ");

        execute!(
            io::stdout(),
            style::SetForegroundColor(Color::White)
        )?;
        print!("{}", self.input_buffer);

        // fix cursor to end
        execute!(
            io::stdout(),
            cursor::MoveTo(self.input_buffer.len() as u16 + 2, current_line)
        )?;

        io::stdout().flush()?;
        Ok(())
    }

    // Process any received data
    fn process_received_data(&mut self) {
        // Try to receive all pending messages
        while let Ok(data) = self.data_receiver.try_recv() {
            match data {
                ReceivedData::Text(text) => {
                    // Process multiline text properly
                    self.display_output(&format!("{}", text), Color::White);
                    let _ = io::stdout().flush();
                }
                ReceivedData::Binary(bytes) => {
                    self.display_output(&format!("{:02X?}", bytes),
                                        Color::DarkYellow);
                }
            }
        }
    }

    pub(crate) fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;

        self.display_output("TDTech Terminal | Type 'help' to get commands list", Color::White);

        let mut running = true;
        while running {
            // Process any received data
            self.process_received_data();

            self.redraw()?;

            // handle keystroke events with a timeout to periodically check for messages
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter => {
                                let command = std::mem::take(&mut self.input_buffer);
                                self.command_history.push(command.clone());
                                self.display_output(&format!("> {}", command), Color::DarkGrey);
                                running = self.process_command(&command);
                            },
                            KeyCode::Char(c) => {
                                self.input_buffer.push(c);
                            },
                            KeyCode::Backspace => {
                                self.input_buffer.pop();
                            },
                            KeyCode::Esc => {
                                running = false;
                            },
                            _ => {} // ignore other keys
                        }
                    }
                }
            }
        }

        // clean up before exit
        terminal::disable_raw_mode()?;
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;

        Ok(())
    }
}