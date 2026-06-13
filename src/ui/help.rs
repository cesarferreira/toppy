use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::theme;

const KEY_COL: usize = 12;
const POPUP_BG: Color = Color::Rgb(16, 18, 28);

pub fn render_help(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(8, 9, 14))),
        area,
    );

    let popup = centered_rect(54, 44, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(
            " toppy — keyboard shortcuts ",
            Style::default()
                .fg(theme::STATUS_KEY_HELP)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::STATUS_KEY_HELP))
        .style(Style::default().bg(POPUP_BG));

    let desc = theme::status_label_style();
    let mut lines = Vec::new();

    lines.extend(section("Navigation", theme::STATUS_KEY_NAV));
    lines.push(row("↑ ↓", "move selection", theme::STATUS_KEY_NAV));
    lines.push(row("PgUp/Dn", "page up / down", theme::STATUS_KEY_NAV));
    lines.push(row("Home/End", "first / last item", theme::STATUS_KEY_NAV));
    lines.push(row("t", "toggle tree view", theme::STATUS_KEY_TREE));
    lines.push(row("c", "toggle CPU per-core view", theme::CPU_BORDER));
    lines.push(row("→ ← Ret", "expand / collapse tree", theme::STATUS_KEY_TREE));

    lines.extend(section("Sorting", theme::STATUS_KEY_SORT));
    lines.push(row("P", "sort by PID", theme::STATUS_KEY_SORT));
    lines.push(row("C", "sort by CPU", theme::STATUS_KEY_SORT));
    lines.push(row("M", "sort by MEM", theme::STATUS_KEY_SORT));
    lines.push(row("T", "sort by command", theme::STATUS_KEY_SORT));

    lines.extend(section("Actions", theme::STATUS_KEY_FILTER));
    lines.push(row("/", "filter processes", theme::STATUS_KEY_FILTER));
    lines.push(row("k", "kill selected process", theme::STATUS_KEY_KILL));
    lines.push(row("r", "force refresh", theme::STATUS_KEY_NAV));
    lines.push(row("+  -", "slower / faster refresh", theme::STATUS_KEY_NAV));

    lines.extend(section("General", theme::STATUS_KEY_HELP));
    lines.push(row("?", "toggle this help", theme::STATUS_KEY_HELP));
    lines.push(row("q", "quit", theme::STATUS_KEY_QUIT));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("        "),
        Span::styled("Press ", desc),
        Span::styled("?", key_style(theme::STATUS_KEY_HELP)),
        Span::styled(" or ", desc),
        Span::styled("Esc", key_style(theme::STATUS_KEY_NAV)),
        Span::styled(" to close", desc),
    ]));

    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

fn key_style(color: Color) -> Style {
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

fn section(title: &'static str, color: Color) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("── ", Style::default().fg(theme::BAR_SEP)),
            Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(" ", Style::default().fg(theme::BAR_SEP)),
            Span::styled(
                "─".repeat(28),
                Style::default().fg(theme::BAR_SEP),
            ),
        ]),
    ]
}

fn row(keys: &'static str, description: &'static str, key_color: Color) -> Line<'static> {
    let pad = KEY_COL.saturating_sub(keys.len());
    Line::from(vec![
        Span::raw("  "),
        Span::styled(keys, key_style(key_color)),
        Span::raw(" ".repeat(pad)),
        Span::styled("│ ", Style::default().fg(theme::BAR_SEP)),
        Span::styled(description, theme::status_label_style()),
    ])
}

pub fn render_kill_menu(frame: &mut Frame, area: Rect, pid: u32) {
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(8, 9, 14))),
        area,
    );

    let popup = centered_rect(44, 28, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(format!(" Kill PID {pid} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::STATUS_KEY_KILL))
        .style(Style::default().bg(POPUP_BG));

    let text = vec![
        Line::from(""),
        row("1", "send SIGTERM (graceful)", theme::STATUS_KEY_KILL),
        row("2", "send SIGKILL (force)", theme::STATUS_KEY_KILL),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", key_style(theme::STATUS_KEY_NAV)),
            Span::styled("  cancel", theme::status_label_style()),
        ]),
    ];

    frame.render_widget(Paragraph::new(text).block(block), popup);
}

pub fn render_filter_input(frame: &mut Frame, area: Rect, filter: &str) {
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(8, 9, 14))),
        area,
    );

    let popup = centered_rect(50, 22, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(
            " Filter ",
            Style::default()
                .fg(theme::STATUS_KEY_FILTER)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::STATUS_KEY_FILTER))
        .style(Style::default().bg(POPUP_BG));

    let text = Line::from(vec![
        Span::raw("  "),
        Span::styled("/", key_style(theme::STATUS_KEY_FILTER)),
        Span::styled(filter, theme::status_message_style()),
        Span::styled("▌", key_style(theme::STATUS_KEY_FILTER)),
    ]);

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
