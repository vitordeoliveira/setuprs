use crossterm::event::KeyCode;

use crate::{
    core::utils::copy_dir_all,
    error::Error,
    tui::app::{App, CurrentMode, DefaultActions},
};

pub struct Confirming<'a> {
    keycode: KeyCode,
    state: &'a mut App,
}

impl<'a> DefaultActions for Confirming<'a> {
    fn exit(&mut self) {}

    fn keycode(&self) -> KeyCode {
        self.keycode
    }

    fn state(&mut self) -> &mut App {
        self.state
    }
}

impl<'a> Confirming<'a> {
    pub fn actions(app: &'a mut App, keycode: KeyCode) -> Self {
        match keycode {
            KeyCode::Char(value) => {
                app.copy_dir_input.push(value);
            }
            KeyCode::Backspace => {
                app.copy_dir_input.pop();
            }
            KeyCode::Enter => {
                if let Some(selected_snapshot) = app.get_selected() {
                    let snapshot_path = format!(
                        "{}{}",
                        app.current_config.snapshots_path, selected_snapshot.id
                    );

                    match copy_dir_all(snapshot_path, app.copy_dir_input.clone()) {
                        Ok(_) => {
                            app.mode = CurrentMode::Exiting;
                        }
                        Err(e) => {
                            app.mode = CurrentMode::Error(Error::IOError(e));
                        }
                    };
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
