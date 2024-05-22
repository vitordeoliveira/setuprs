use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{
    style::{palette::tailwind, Stylize},
    widgets::ListItem,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

use super::{ui::ui, Tui};

struct EventHandler {
    rx: tokio::sync::mpsc::UnboundedReceiver<KeyCode>,
    stop_cancellation_token: CancellationToken,
}

#[derive(Debug, Default)]
pub struct App {
    pub left_size: u16,
    pub list: Vec<ObjList>,
    pub current_item: String,
}

#[derive(Debug, Default)]
pub struct ObjList {
    pub id: String,
    pub selected: bool,
}

impl ObjList {
    pub fn from_array(arr: &[&str]) -> Vec<Self> {
        arr.iter()
            .map(|id| ObjList {
                id: id.to_string(),
                selected: false,
            })
            .collect()
    }

    pub fn to_list_item(&self, current_item: String) -> ListItem {
        match self.id == current_item {
            true => ListItem::new(self.id.to_string()).bg(tailwind::GREEN.c400),
            false => ListItem::new(self.id.to_string()),
        }
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
    pub fn new(list: Vec<ObjList>) -> Result<Self> {
        Ok(App {
            current_item: list[0].id.clone(),
            list,
            left_size: 50,
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
                KeyCode::Down => self.change_current_down(),
                KeyCode::Up => self.change_current_up(),
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

    pub fn change_current_down(&mut self) {
        // let current_position = self
        //     .list
        //     .iter()
        //     .position(|item| item.id == self.current_item);
        //
        // match current_position.unwrap() == self.list.len() - 1 {
        //     true => self.current_item = self.list[0].id.clone(),
        //     false => self.current_item = self.list[1 + current_position.unwrap()].id.clone(),
        // }
        match self
            .list
            .iter()
            .position(|item| item.id == self.current_item)
        {
            Some(current_position) if current_position == self.list.len() - 1 => {
                self.current_item = self.list[0].id.clone()
            }
            Some(current_position) => {
                self.current_item = self.list[1 + current_position].id.clone()
            }
            None => {}
        }
    }

    fn change_current_up(&mut self) {
        match self
            .list
            .iter()
            .position(|item| item.id == self.current_item)
        {
            Some(0) => self.current_item = self.list[self.list.len() - 1].id.clone(),
            Some(current_position) => {
                self.current_item = self.list[current_position - 1].id.clone()
            }
            None => {}
        }
    }
}
