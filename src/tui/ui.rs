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

    if state.left_size > 50 {
        let block = Block::bordered().title("Popup");
        let area = centered_rect(60, 60, f.size());

        f.render_widget(Clear, area);
        f.render_widget(block, area);
    }

    if let CurrentMode::Confirming = state.mode {
        let block = Block::bordered().title("Where should the copy being made?");
        let area = centered_rect(60, 60, f.size());

        let style = Style::default().fg(Color::Blue).bg(Color::White);
        let input = Paragraph::new(Text::styled(
            format!(" {} ", state.copy_dir_input.clone()),
            style,
        ))
        .block(block.padding(Padding::top(area.height / 2)))
        .centered();

        f.render_widget(Clear, area);
        f.render_widget(input, area);
    }

    if let CurrentMode::Exiting = state.mode {
        let area = centered_rect(30, 30, f.size());

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(centered_rect(80, 80, area));

        let block = Block::bordered().title("Are you sure you want to quit?");

        let style = Style::default().fg(Color::Blue).bg(Color::White);
        let yes_button = Paragraph::new(Text::styled(" Yes (y/Y) ", style))
            .block(Block::new().padding(Padding::top(inner_layout[0].height / 2)))
            .centered();

        let no_button = Paragraph::new(Text::styled(" No (n/N) ", style))
            .block(Block::new().padding(Padding::top(inner_layout[0].height / 2)))
            .centered();

        f.render_widget(Clear, area);
        f.render_widget(block, area);
        f.render_widget(yes_button, inner_layout[0]);
        f.render_widget(no_button, inner_layout[1]);
    };
}
