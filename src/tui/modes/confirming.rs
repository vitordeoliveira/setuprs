use std::env;

use crossterm::event::KeyCode;

use crate::{
    core::utils::copy_dir_all,
    tui::app::{App, CurrentMode, Exit},
};

pub struct Confirming<'a> {
    keycode: KeyCode,
    state: &'a mut App,
}

impl<'a> Exit for Confirming<'a> {
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
            KeyCode::Char('e') => app.mode = CurrentMode::Exiting,
            KeyCode::Enter => {
                // TODO: Create popup asking the name of the folder where the copy will be "."
                // if no folder is needed, (default) is the snapshot tag name or id or
                // setupr.toml default name
                if let (Ok(path), Some(selected_snapshot)) =
                    (env::current_dir(), app.get_selected())
                {
                    let snapshot_path = format!(
                        "{}{}",
                        app.current_config.snapshots_path, selected_snapshot.id
                    );

                    match copy_dir_all(snapshot_path, path) {
                        Ok(_) => {
                            app.mode = CurrentMode::Exiting;
                        }
                        // TODO: when error, show error popup and reset to initial state
                        Err(_) => println!("error"),
                    };
                }
            }
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
