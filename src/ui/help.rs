use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::theme;

pub fn render_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(70, 70, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(theme::STATUS_KEY_HELP)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::STATUS_KEY_HELP));

    let key = |color: Color| Style::default().fg(color).add_modifier(Modifier::BOLD);
    let desc = theme::status_action_style();
    let section = |title: &'static str, color: Color| {
        Line::from(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
    };

    let text = vec![
        Line::from(""),
        section("Navigation", theme::STATUS_KEY_NAV),
        binding(
            vec!["↑", "↓", "PgUp", "PgDn", "Home", "End"],
            "move selection",
            theme::STATUS_KEY_NAV,
        ),
        binding(vec!["t"], "toggle tree view", theme::STATUS_KEY_TREE),
        Line::from(""),
        section("Sorting", theme::STATUS_KEY_SORT),
        binding(vec!["P"], "sort by PID", theme::STATUS_KEY_SORT),
        binding(vec!["C"], "sort by CPU", theme::STATUS_KEY_SORT),
        binding(vec!["M"], "sort by MEM", theme::STATUS_KEY_SORT),
        binding(vec!["T"], "sort by Command", theme::STATUS_KEY_SORT),
        Line::from(""),
        section("Filter & actions", theme::STATUS_KEY_FILTER),
        binding(vec!["/"], "filter processes (Esc to clear)", theme::STATUS_KEY_FILTER),
        binding(vec!["k"], "kill menu (TERM / KILL)", theme::STATUS_KEY_KILL),
        binding(vec!["→", "←", "Enter"], "expand/collapse tree node", theme::STATUS_KEY_TREE),
        binding(vec!["r"], "force refresh", theme::STATUS_KEY_NAV),
        binding(vec!["+", "-"], "slower / faster refresh", theme::STATUS_KEY_NAV),
        Line::from(""),
        section("General", theme::STATUS_KEY_HELP),
        binding(vec!["?"], "toggle this help", theme::STATUS_KEY_HELP),
        binding(vec!["q"], "quit", theme::STATUS_KEY_QUIT),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", desc),
            Span::styled("?", key(theme::STATUS_KEY_HELP)),
            Span::styled(" or ", desc),
            Span::styled("Esc", key(theme::STATUS_KEY_NAV)),
            Span::styled(" to close", desc),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, popup);
}

fn binding(keys: Vec<&'static str>, action: &'static str, key_color: Color) -> Line<'static> {
    let key_style = Style::default().fg(key_color).add_modifier(Modifier::BOLD);
    let desc_style = theme::status_action_style();

    let mut spans = vec![Span::raw("  ")];
    for (idx, key) in keys.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::styled("/", desc_style));
        }
        spans.push(Span::styled(*key, key_style));
    }
    spans.push(Span::styled(format!("  {action}"), desc_style));
    Line::from(spans)
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
