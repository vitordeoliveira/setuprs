use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::widgets::{ListItem, ListState};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::core::{utils::confirm_selection, Config};

use super::{ui::ui, Tui};

struct EventHandler {
    rx: tokio::sync::mpsc::UnboundedReceiver<KeyCode>,
    stop_cancellation_token: CancellationToken,
}

#[derive(Debug, Default)]
pub struct App {
    pub current_config: Config,
    pub left_size: u16,
    pub list: Vec<ObjList>,
    // pub current_item: String,
    pub list_state: ListState,
    pub last_selected: Option<usize>,
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

    pub fn to_list_item(&self, current_item: usize) -> ListItem {
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

    fn stop(&self) {
        self.stop_cancellation_token.cancel();
    }
}

#[allow(dead_code)]
impl App {
    pub fn new(list: Vec<ObjList>, current_config: Config) -> Result<Self> {
        Ok(App {
            // current_item: list[0].id.clone(),
            current_config,
            list_state: ListState::default().with_selected(Some(0)),
            list,
            left_size: 50,
            last_selected: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;
        let mut events = EventHandler::new();
        loop {
            tui.terminal.draw(|f| ui(f, self))?;
            match events.next().await.unwrap() {
                KeyCode::Char('q') => {
                    events.stop();
                    break;
                }
                KeyCode::Enter => confirm_selection(),
                KeyCode::Down => self.next(),
                KeyCode::Up => self.previous(),
                KeyCode::Right => self.left_size += 1,
                KeyCode::Left => {
                    if self.left_size > 0 {
                        self.left_size -= 1
                    }
                }
                _ => {}
            }
        }

        tui.exit()?;
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

    fn previous(&mut self) {
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
}
