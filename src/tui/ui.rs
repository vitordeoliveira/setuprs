use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::App;

#[allow(dead_code)]
pub fn ui(f: &mut Frame, state: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(10)
        .constraints([Constraint::Percentage(state.left_size), Constraint::Min(1)])
        .split(f.size());

    let instructions = Text::from(vec![
        Line::from(vec!["Decrement Menu Size".into(), "<Left>".blue().bold()]),
        Line::from(vec!["Increment Menu Size".into(), "<Right>".blue().bold()]),
        "Quit <Q>".into(),
    ]);

    // let block = Block::default().title(instructions);
    f.render_widget(instructions, chunks[0]);
    let text = format!("size: {}", state.left_size);
    f.render_widget(Paragraph::new(text).white().on_blue(), chunks[1]);
}
