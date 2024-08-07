use std::{env, panic};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::widgets::{ListItem, ListState};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{core::Config, error::Result};

use super::{
    modes::{confirming::Confirming, errormode::ErrorMode, main::Main},
    ui::ui,
    Tui,
};

pub struct EventHandler {
    rx: tokio::sync::mpsc::UnboundedReceiver<KeyCode>,
    stop_cancellation_token: CancellationToken,
}

pub trait DefaultActions {
    fn exit(&mut self) {
        if let KeyCode::Char('q') = self.keycode() {
            self.state().mode = CurrentMode::Exiting;
        }
    }
    fn escape(&mut self) {
        if let KeyCode::Esc = self.keycode() {
            self.state().mode = CurrentMode::Main(Content::Help);
        }
    }
    fn keycode(&self) -> KeyCode;
    fn state(&mut self) -> &mut App;
}

struct Action<T: ?Sized + DefaultActions>(Box<T>);

impl<T: ?Sized + DefaultActions> Action<T> {
    fn run(&mut self) {
        self.0.exit();
        self.0.escape();
    }
}

#[derive(Debug, Default)]
pub struct App {
    pub current_config: Config,
    pub left_size: u16,
    pub list: Vec<ObjList>,
    pub list_state: ListState,
    pub last_selected: Option<usize>,
    pub mode: CurrentMode,
    pub copy_dir_input: String,
}

#[derive(Debug)]
pub enum Content {
    Help,
}

#[derive(Debug)]
pub enum CurrentMode {
    Main(Content),
    Confirming,
    Exiting,
    Error(crate::error::Error),
}

impl Default for CurrentMode {
    fn default() -> Self {
        CurrentMode::Main(Content::Help)
    }
}

#[derive(Debug, Default)]
pub struct ObjList {
    pub id: String,
    pub selected: bool,
}

impl ObjList {
    #[allow(dead_code)]
    pub fn from_array(arr: Vec<String>) -> Vec<Self> {
        arr.iter()
            .map(|id| ObjList {
                id: id.to_string(),
                selected: false,
            })
            .collect()
    }

    pub fn to_list_item(&self) -> ListItem {
        // match self.id == current_item {
        //     true => ListItem::new(self.id.to_string()).bg(tailwind::GREEN.c400),
        //     false => ListItem::new(self.id.to_string()),
        // }
        ListItem::new(self.id.to_string())
    }
}

impl EventHandler {
    fn new() -> Self {
        let tick_rate = std::time::Duration::from_millis(250);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let stop_cancellation_token = CancellationToken::new();
        let _stop_cancellation_token = stop_cancellation_token.clone();
        tokio::spawn(async move {
            loop {
                select! {
                    _ = _stop_cancellation_token.cancelled() => {
                        break;
                    }

                    _ = async {
                if event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        if key.kind == KeyEventKind::Press {
                            let _ = tx.send(key.code);
                        };
                    }
                }
                } => {}
                }
            }
        });

        EventHandler {
            rx,
            stop_cancellation_token,
        }
    }

    async fn next(&mut self) -> Option<KeyCode> {
        self.rx.recv().await
    }

    pub fn stop(&self) {
        self.stop_cancellation_token.cancel();
    }
}

#[allow(dead_code)]
impl App {
    pub fn new(list: Vec<ObjList>, current_config: Config) -> Result<Self> {
        Ok(App {
            current_config,
            list_state: ListState::default().with_selected(Some(0)),
            list,
            left_size: 50,
            copy_dir_input: env::current_dir()?.display().to_string(),
            ..App::default()
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut events = EventHandler::new();
        let canceltoken = events.stop_cancellation_token.clone();
        panic::set_hook(Box::new(move |message| {
            Tui::exit().unwrap();
            canceltoken.cancel();
            println!("{message}");
        }));

        let mut tui = Tui::new()?;
        tui.enter()?;
        loop {
            tui.terminal.draw(|f| ui(f, self))?;

            if let Some(keycode) = events.next().await {
                let action: Option<Action<dyn DefaultActions>> = match &self.mode {
                    CurrentMode::Main(_) => Some(Action(Box::new(Main::actions(self, keycode)))),
                    CurrentMode::Confirming => {
                        Some(Action(Box::new(Confirming::actions(self, keycode))))
                    }
                    CurrentMode::Exiting => match keycode {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            events.stop();
                            break;
                        }
                        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                            self.mode = CurrentMode::Main(Content::Help);
                            None
                        }
                        _ => None,
                    },
                    CurrentMode::Error(_) => {
                        Some(Action(Box::new(ErrorMode::actions(self, keycode))))
                    }
                };

                if let Some(mut action) = action {
                    action.run();
                } else if let KeyCode::Char('q') = keycode {
                    drop(action);
                    self.mode = CurrentMode::Exiting;
                }
            }
        }

        Tui::exit()?;
        Ok(())
    }

    pub fn next(&mut self) {
        // let current_position = self
        //     .list
        //     .iter()
        //     .position(|item| item.id == self.current_item);
        //
        // match current_position.unwrap() == self.list.len() - 1 {
        //     true => self.current_item = self.list[0].id.clone(),
        //     false => self.current_item = self.list[1 + current_position.unwrap()].id.clone(),
        // }
        // match self
        //     .list
        //     .iter()
        //     .position(|item| item.id == self.current_item)
        // {
        //     Some(current_position) if current_position == self.list.len() - 1 => {
        //         self.current_item = self.list[0].id.clone()
        //     }
        //     Some(current_position) => {
        //         self.current_item = self.list[1 + current_position].id.clone()
        //     }
        //     None => {}
        // }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        // match self
        //     .list
        //     .iter()
        //     .position(|item| item.id == self.current_item)
        // {
        //     Some(0) => self.current_item = self.list[self.list.len() - 1].id.clone(),
        //     Some(current_position) => {
        //         self.current_item = self.list[current_position - 1].id.clone()
        //     }
        //     None => {}
        // }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.list.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.list_state.select(Some(i));
    }

    pub fn get_selected(&self) -> Option<&ObjList> {
        let index = self.list_state.selected()?;
        self.list.get(index)
    }
}
