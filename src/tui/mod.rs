use color_eyre::eyre::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend as Backend;

pub mod app;
mod modes;
pub mod ui;
#[allow(dead_code)]
pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
}

#[allow(dead_code)]
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
