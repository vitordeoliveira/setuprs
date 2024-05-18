use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};

use super::{ui::ui, Tui};

#[derive(Debug, Default)]
pub struct App {
    pub left_size: u16,
}

#[allow(dead_code)]
impl App {
    pub fn run() -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        let mut app = App::default();
        loop {
            tui.terminal.draw(|f| ui(f, &mut app))?;
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Right => app.left_size += 1,
                        KeyCode::Left => {
                            if app.left_size > 0 {
                                app.left_size -= 1
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        tui.exit()?;
        Ok(())
    }
}
