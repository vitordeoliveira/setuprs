use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};

use super::{ui::ui, Tui};

struct EventHandler {
    rx: tokio::sync::mpsc::UnboundedReceiver<KeyCode>,
}

#[derive(Debug, Default)]
pub struct App {
    pub left_size: u16,
}

impl EventHandler {
    fn new() -> Self {
        let tick_rate = std::time::Duration::from_millis(250);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        if key.kind == KeyEventKind::Press {
                            let _ = tx.send(key.code);
                        };
                    }
                }
                println!("I am still here");
            }
        });

        EventHandler { rx }
    }

    async fn next(&mut self) -> Option<KeyCode> {
        self.rx.recv().await
    }
}

#[allow(dead_code)]
impl App {
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;
        let mut events = EventHandler::new();
        loop {
            tui.terminal.draw(|f| ui(f, self))?;
            match events.next().await.unwrap() {
                KeyCode::Char('q') => break,
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

    async fn events() -> Result<()> {
        Ok(())
    }
}
