use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::theme;

pub fn render_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(70, 70, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::HEADER_FG));

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ↑/↓, PgUp/PgDn, Home/End  move selection"),
        Line::from("  t                        toggle tree view"),
        Line::from(""),
        Line::from(Span::styled("Sorting", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  P  sort by PID"),
        Line::from("  C  sort by CPU"),
        Line::from("  M  sort by MEM"),
        Line::from("  T  sort by Command"),
        Line::from(""),
        Line::from(Span::styled("Filter & actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /   start filter (type to search, Esc to clear)"),
        Line::from("  k   kill menu (TERM / KILL)"),
        Line::from("  F9  kill menu"),
        Line::from("  →/← or Enter  expand/collapse tree node"),
        Line::from("  r   force refresh"),
        Line::from("  + / -  slower / faster refresh (htop-style)"),
        Line::from(""),
        Line::from(Span::styled("General", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ? / F1  toggle this help"),
        Line::from("  q       quit"),
        Line::from(""),
        Line::from(Span::styled("Press ? or Esc to close", theme::dim_style())),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, popup);
}

pub fn render_kill_menu(frame: &mut Frame, area: Rect, pid: u32) {
    let popup = centered_rect(50, 30, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(format!(" Kill PID {pid} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::utilization_color(80.0)));

    let text = vec![
        Line::from(""),
        Line::from("  1  Send SIGTERM (graceful)"),
        Line::from("  2  Send SIGKILL (force)"),
        Line::from(""),
        Line::from(Span::styled("  Esc  cancel", theme::dim_style())),
    ];

    frame.render_widget(Paragraph::new(text).block(block), popup);
}

pub fn render_filter_input(frame: &mut Frame, area: Rect, filter: &str) {
    let popup = centered_rect(60, 20, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Filter ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::CPU_BORDER));

    let text = format!("/{filter}▌");
    frame.render_widget(Paragraph::new(text).block(block), popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
