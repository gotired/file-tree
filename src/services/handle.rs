use crate::repository::{App, Handle, Node};
use crate::types::{InputMode, TuiTerminal};

use std::{
    error::Error,
    io::{self, IsTerminal, stdout},
};

use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

impl Handle {
    /// MODE 1: CLI Mode
    /// Reads from stdin (blocking), generates tree, prints to stdout.
    pub fn cli() -> Result<(), Box<dyn Error>> {
        let mut root = Node::default();
        let stdin = io::stdin();
        let mut count = 0;

        // Read lines from stdin
        for line in stdin.lines() {
            let path = line?;
            let trimmed = path.trim();
            if !trimmed.is_empty() {
                let parts: Vec<&str> = trimmed.split('/').filter(|s| !s.is_empty()).collect();
                if !parts.is_empty() {
                    root.insert(&parts);
                    count += 1;
                }
            }
        }

        // Only print if we actually got data
        if count > 0 {
            let mut output_lines = Vec::new();
            root.generate_tree_data("", "", &mut output_lines);
            for (line, _) in output_lines {
                println!("{}", line);
            }
        } else {
            // If ran interactively without pipe/args, show usage
            if io::stdin().is_terminal() {
                eprintln!("Usage: ");
                eprintln!("  1. Pipe data: cat files.txt | file-tree");
                eprintln!("  2. TUI Mode:  file-tree --tui");
            }
        }

        Ok(())
    }

    /// MODE 2: TUI Mode
    /// Launches the interactive interface.
    pub fn tui() -> Result<(), Box<dyn Error>> {
        let mut initial_paths = Vec::new();

        // Check if data is being piped IN, even in TUI mode
        if !io::stdin().is_terminal() {
            let stdin = io::stdin();
            for line in stdin.lines() {
                if let Ok(l) = line {
                    if !l.trim().is_empty() {
                        initial_paths.push(l);
                    }
                }
            }
        } else {
            // Default sample if no input provided
            initial_paths = vec![];
        }

        let mut app = App::new(initial_paths);

        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;

        let res = Self::run_app(&mut terminal, &mut app);

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }

    fn run_app(terminal: &mut TuiTerminal, app: &mut App) -> io::Result<()> {
        loop {
            terminal.draw(|f| Self::draw_ui(f, app))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                match app.mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('i') => {
                            app.mode = InputMode::Insert;
                            app.status_message = "INSERT MODE: Type or Paste paths.".to_string();
                        }
                        KeyCode::Char('e') => app.edit_selected(),
                        KeyCode::Char('d') | KeyCode::Delete => app.initiate_delete(),
                        KeyCode::Char('c') => app.copy_to_clipboard(),
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        _ => {}
                    },
                    InputMode::DeleteConfirm => match key.code {
                        KeyCode::Char('d') | KeyCode::Delete => app.confirm_delete(),
                        _ => app.cancel_delete(),
                    },
                    InputMode::Insert => match key.code {
                        KeyCode::Enter => app.add_path(),
                        KeyCode::Char(c) => app.input_buffer.push(c),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Esc => {
                            app.mode = InputMode::Normal;
                            app.input_buffer.clear();
                            app.status_message = "Done inserting.".to_string();
                        }
                        _ => {}
                    },
                    InputMode::Edit => match key.code {
                        KeyCode::Enter => app.add_path(),
                        KeyCode::Char(c) => app.input_buffer.push(c),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Esc => app.cancel_edit(),
                        _ => {}
                    },
                }
            }
        }
    }

    // --- UI Rendering ---

    fn draw_ui(frame: &mut ratatui::Frame, app: &mut App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(frame.size());

        let items: Vec<ListItem> = app
            .tree_items
            .iter()
            .map(|(line, _)| ListItem::new(line.as_str()))
            .collect();

        let tree_block = Block::default()
            .borders(Borders::ALL)
            .title(" Project Structure ")
            .style(Style::default());

        let list = List::new(items)
            .block(tree_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[0], &mut app.list_state);

        let help_text = match app.mode {
            InputMode::Normal => "NORMAL: [i]nsert (Batch) | [e]dit | [d]elete | [c]opy | [q]uit",
            InputMode::Insert => "INSERT: Type/Paste paths | [Enter] Add & Next | [Esc] Done",
            InputMode::Edit => "EDIT: Modify path | [Enter] Save (Renames children) | [Esc] Cancel",
            InputMode::DeleteConfirm => "CONFIRM: [d] to Delete | [Esc/Any] to Cancel",
        };

        let status_color = match app.mode {
            InputMode::DeleteConfirm => Color::Red,
            InputMode::Edit => Color::Magenta,
            _ => Color::Cyan,
        };

        let status_text = format!("Status: {} | {}", app.status_message, help_text);
        let status_p = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .style(Style::default().fg(status_color));

        frame.render_widget(status_p, chunks[1]);

        let input_style = match app.mode {
            InputMode::Normal => Style::default().fg(Color::DarkGray),
            _ => Style::default().fg(Color::Yellow),
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(match app.mode {
                InputMode::Normal => " Input Path ",
                InputMode::Insert => " Batch Insert (Paste here) ",
                InputMode::Edit => " Editing Path ",
                InputMode::DeleteConfirm => " Input Path (Disabled) ",
            })
            .border_style(input_style);

        let input_p = Paragraph::new(app.input_buffer.as_str())
            .block(input_block)
            .style(match app.mode {
                InputMode::Normal => Style::default(),
                _ => Style::default().fg(Color::White),
            });

        frame.render_widget(input_p, chunks[2]);

        if app.mode == InputMode::Insert || app.mode == InputMode::Edit {
            frame.set_cursor(
                chunks[2].x + 1 + app.input_buffer.len() as u16,
                chunks[2].y + 1,
            )
        }
    }
}
