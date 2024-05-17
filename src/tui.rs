use color_eyre::eyre::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend as Backend;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};

use crate::App;

pub fn ui(f: &mut Frame, state: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(10)
        .constraints([Constraint::Percentage(state.left_size), Constraint::Min(1)])
        .split(f.size());

    let instructions = Text::from(vec![
        Line::from(vec!["Decrement Menu Size".into(), "<Left>".blue().bold()]),
        Line::from(vec!["Increment Menu Size".into(), "<Right>".blue().bold()]),
        "Quit <Q>".into(),
    ]);

    // let block = Block::default().title(instructions);
    f.render_widget(instructions, chunks[0]);
    let text = format!("size: {}", state.left_size);
    f.render_widget(Paragraph::new(text).white().on_blue(), chunks[1]);
}

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr()))?;
        Ok(Self { terminal })
    }

    pub fn enter(&self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            std::io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        Ok(())
    }

    pub fn exit(&self) -> Result<()> {
        crossterm::execute!(
            std::io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }
}
