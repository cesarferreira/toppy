use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{app::App, theme};

pub fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let cpu = if app.warmed_up {
        format!("{:.1}%", app.cpu.global)
    } else {
        "…".to_string()
    };

    let mut spans = vec![
        Span::styled("CPU ", theme::status_label_style()),
        Span::styled(cpu, theme::utilization_style(app.cpu.global)),
        Span::styled(
            format!("  {}ms  ", app.refresh_ms),
            theme::status_label_style(),
        ),
    ];

    if let Some(msg) = &app.status_message {
        spans.push(Span::styled(msg.clone(), theme::status_message_style()));
        spans.push(Span::styled("  ", theme::status_action_style()));
    }

    spans.extend(shortcut_spans());

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn shortcut_spans() -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    shortcut(&mut spans, "?", "", "", "help", theme::STATUS_KEY_HELP);
    shortcut(&mut spans, "c", "", "", "cores", theme::CPU_BORDER);
    shortcut(&mut spans, "+", "-", "", "refresh", theme::STATUS_KEY_NAV);
    shortcut(&mut spans, "P", "C", "M", "sort", theme::STATUS_KEY_SORT);
    shortcut(&mut spans, "/", "", "", "filter", theme::STATUS_KEY_FILTER);
    shortcut(&mut spans, "t", "", "", "tree", theme::STATUS_KEY_TREE);
    shortcut(&mut spans, "k", "", "", "kill", theme::STATUS_KEY_KILL);
    shortcut(&mut spans, "r", "", "", "refresh", theme::STATUS_KEY_NAV);
    shortcut(&mut spans, "q", "", "", "quit", theme::STATUS_KEY_QUIT);
    spans
}

fn shortcut(
    spans: &mut Vec<Span<'static>>,
    key1: &str,
    key2: &str,
    key3: &str,
    action: &str,
    key_color: Color,
) {
    let key_style = Style::default().fg(key_color).add_modifier(Modifier::BOLD);
    let sep_style = theme::status_action_style();

    if !spans.is_empty() {
        spans.push(Span::styled("  ", sep_style));
    }

    spans.push(Span::styled(key1.to_string(), key_style));
    if !key2.is_empty() {
        spans.push(Span::styled(key2.to_string(), key_style));
    }
    if !key3.is_empty() {
        spans.push(Span::styled(key3.to_string(), key_style));
    }
    spans.push(Span::styled(format!(" {action}"), sep_style));
}
