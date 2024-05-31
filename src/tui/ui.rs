use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{palette::tailwind, Stylize},
    text::Line,
    Frame,
};

use ratatui::{prelude::*, widgets::*};

use crate::tui::app::App;

use super::app::CurrentMode;

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

    let items: Vec<ListItem> = state.list.iter().map(|item| item.to_list_item()).collect();

    let list = List::new(items)
        .block(Block::default().title("List").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(tailwind::GREEN.c400))
        .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(list, chunks[0], &mut state.list_state);
    f.render_widget(help_instructions, chunks[1]);

    if let CurrentMode::Exiting = state.mode {
        let block = Block::bordered().title("Are you sure?");
        let area = centered_rect(60, 60, f.size());

        f.render_widget(Clear, area);
        f.render_widget(block, area);
    };

    if state.left_size > 50 {
        let block = Block::bordered().title("Popup");
        let area = centered_rect(60, 60, f.size());

        f.render_widget(Clear, area);
        f.render_widget(block, area);
    }
}
