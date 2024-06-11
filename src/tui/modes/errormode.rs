use crossterm::event::KeyCode;

use crate::tui::app::{App, Content, CurrentMode, DefaultActions};

pub struct ErrorMode<'a> {
    keycode: KeyCode,
    state: &'a mut App,
}

impl<'a> DefaultActions for ErrorMode<'a> {
    fn keycode(&self) -> KeyCode {
        self.keycode
    }

    fn state(&mut self) -> &mut App {
        self.state
    }
}

impl<'a> ErrorMode<'a> {
    pub fn actions(app: &'a mut App, keycode: KeyCode) -> Self {
        if let KeyCode::Char(_) = keycode {
            app.mode = CurrentMode::Main(Content::Help)
        }
        Self {
            keycode,
            state: app,
        }
    }
}
