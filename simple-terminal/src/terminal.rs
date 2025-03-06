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

    // 显示内容的函数，支持彩色输出
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
                self.display_output("available commands:", Color::Yellow);
                self.display_output("  help", Color::Yellow);
                self.display_output("  clear", Color::Yellow);
                self.display_output("  exit", Color::Yellow);
            }
            "" => {}
            _ => {
                unimplemented!("at command")
            }
        }
        true
    }

    // 重新绘制整个终端界面
    fn redraw(&self) -> io::Result<()> {
        execute!(io::stdout(), terminal::Clear(ClearType::All))?;

        // 显示历史输出
        for (i, (message, color)) in self.output_history.iter().enumerate() {
            execute!(
                io::stdout(), 
                cursor::MoveTo(0, i as u16),
                style::SetForegroundColor(*color)
            )?;
            print!("{}", message);
        }

        // 显示提示符和当前输入
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

        // 将光标定位到输入的末尾
        execute!(
            io::stdout(),
            cursor::MoveTo(self.input_buffer.len() as u16 + 2, prompt_line)
        )?;

        io::stdout().flush()?;
        Ok(())
    }

    // 运行终端
    pub(crate) fn run(&mut self) -> io::Result<()> {
        // 进入备用屏幕并设置原始模式
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;

        // 显示欢迎信息
        self.display_output("Simple Terminal | Type 'help' to get help", Color::Blue);

        let mut running = true;
        while running {
            // 绘制界面
            self.redraw()?;

            // 处理事件
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
                        _ => {} // 忽略其他按键
                    }
                }
            }
        }

        // 退出前的清理工作
        terminal::disable_raw_mode()?;
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;

        Ok(())
    }
}