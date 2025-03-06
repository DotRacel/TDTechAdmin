mod terminal;

use std::io;
use crate::terminal::Terminal;

fn main() -> io::Result<()> {
    let mut terminal = Terminal::new();
    terminal.run()
}