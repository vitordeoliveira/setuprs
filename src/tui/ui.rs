use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::{Line, Text},
    Frame,
};

use ratatui::{prelude::*, widgets::*};

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

    let items: Vec<ListItem> = state
        .list
        .iter()
        .map(|i| i.to_list_item(state.current_item.clone()).clone())
        .collect();
    let list = List::new(items)
        .block(Block::default().title("List").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true);

    f.render_widget(list, chunks[0]);

    // let block = Block::default().title(instructions);
    f.render_widget(instructions, chunks[1]);
    // let text = format!("size: {}", state.left_size);
    // f.render_widget(Paragraph::new(text).white().on_blue(), chunks[1]);
}
