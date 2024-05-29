use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{palette::tailwind, Stylize},
    text::Line,
    Frame,
};

use ratatui::{prelude::*, widgets::*};

use crate::tui::app::App;

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

pub fn ui(f: &mut Frame, state: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(10)
        .constraints([Constraint::Percentage(state.left_size), Constraint::Min(1)])
        .split(f.size());

    let content = Block::default().borders(Borders::ALL);
    let help_instructions = Paragraph::new(vec![
        Line::from(vec![
            "Select the setup you want".into(),
            "<UP> or <DOWN>".blue().bold(),
        ]),
        Line::from(vec!["Confirm".into(), "<ENTER>".blue().bold()]),
        Line::from(vec!["Increment Menu Size".into(), "<Right>".blue().bold()]),
        "Quit <Q>".into(),
    ])
    .block(content);

        .list
        .iter()
        .enumerate()
        .map(|(i, item)| item.to_list_item(i))
        .collect();

    let list = List::new(items)
        .block(Block::default().title("List").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(tailwind::GREEN.c400))
        .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(list, chunks[0], &mut state.list_state);

    // let block = Block::default().title(instructions);
    f.render_widget(help_instructions, chunks[1]);
    // let text = format!("size: {}", state.left_size);
    // f.render_widget(Paragraph::new(text).white().on_blue(), chunks[1]);
}
