use crossterm::event::KeyCode;

use crate::tui::app::{App, CurrentMode, DefaultActions};

pub struct Main<'a> {
    keycode: KeyCode,
    state: &'a mut App,
}

impl<'a> DefaultActions for Main<'a> {
    fn keycode(&self) -> KeyCode {
        self.keycode
    }

    fn state(&mut self) -> &mut App {
        self.state
    }
}

impl<'a> Main<'a> {
    pub fn actions(app: &'a mut App, keycode: KeyCode) -> Self {
        match keycode {
            KeyCode::Char('e') => app.mode = CurrentMode::Exiting,
            KeyCode::Enter => app.mode = CurrentMode::Confirming,
            KeyCode::Down => app.next(),
            KeyCode::Up => app.previous(),
            KeyCode::Right => app.left_size += 1,
            KeyCode::Left => {
                if app.left_size > 0 {
                    app.left_size -= 1
                }
            }
            _ => {}
        };

        Self {
            keycode,
            state: app,
        }
    }
}
