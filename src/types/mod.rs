use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
    Edit,
    DeleteConfirm,
}

pub type TuiTerminal = Terminal<CrosstermBackend<io::Stdout>>;
