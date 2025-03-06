use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{self, Color, Stylize},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

pub struct Terminal {
    input_buffer: String,
    command_history: Vec<String>,
    output_history: Vec<(String, Color)>,
}

impl Terminal {
    pub(crate) fn new() -> Self {
        Self {
            input_buffer: String::new(),
            command_history: Vec::new(),
            output_history: Vec::new(),
        }
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
                unimplemented!("at command")
            }
        }
        true
    }

    fn redraw(&self) -> io::Result<()> {
        execute!(io::stdout(), terminal::Clear(ClearType::All))?;

        // show history
        for (i, (message, color)) in self.output_history.iter().enumerate() {
            execute!(
                io::stdout(), 
                cursor::MoveTo(0, i as u16),
                style::SetForegroundColor(*color)
            )?;
            print!("{}", message);
        }

        // show prompt
        let prompt_line = self.output_history.len() as u16;
        execute!(
            io::stdout(),
            cursor::MoveTo(0, prompt_line),
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
            cursor::MoveTo(self.input_buffer.len() as u16 + 2, prompt_line)
        )?;

        io::stdout().flush()?;
        Ok(())
    }

    pub(crate) fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;

        self.display_output("TDTech Terminal | Type 'help' to get commands list", Color::White);

        let mut running = true;
        while running {
            self.redraw()?;

            // handle keystroke events
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

        // clean up before exit
        terminal::disable_raw_mode()?;
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;

        Ok(())
    }
}