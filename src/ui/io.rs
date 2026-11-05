use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::metrics::{format::format_rate, io::IoSnapshot};
use crate::theme::{self, IO_BORDER, IO_READ, IO_RX, IO_TX, IO_WRITE};

pub fn render_io(frame: &mut Frame, area: Rect, io: &IoSnapshot) {
    let block = Block::default()
        .title(" IO ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(IO_BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    if !io.ready {
        frame.render_widget(
            Paragraph::new("collecting disk and network IO...").style(theme::dim_style()),
            inner,
        );
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); inner.height as usize])
        .split(inner);

    frame.render_widget(
        Paragraph::new(rate_line(
            "Disk",
            "R",
            &format_rate(io.disk_read),
            IO_READ,
            "W",
            &format_rate(io.disk_write),
            IO_WRITE,
        )),
        rows[0],
    );
    if rows.len() > 1 {
        frame.render_widget(
            Paragraph::new(rate_line(
                "Net",
                "↓",
                &format_rate(io.net_rx),
                IO_RX,
                "↑",
                &format_rate(io.net_tx),
                IO_TX,
            )),
            rows[1],
        );
    }
}

fn rate_line(
    kind: &str,
    left_label: &str,
    left: &str,
    left_color: ratatui::style::Color,
    right_label: &str,
    right: &str,
    right_color: ratatui::style::Color,
) -> Line<'static> {
    let value = theme::status_label_style().add_modifier(ratatui::style::Modifier::BOLD);
    Line::from(vec![
        Span::styled(format!("{kind:<4} "), theme::meter_label_style()),
        Span::styled(format!("{left_label} "), theme::status_label_style()),
        Span::styled(format!("{left:<12}"), value.fg(left_color)),
        Span::styled(format!("{right_label} "), theme::status_label_style()),
        Span::styled(right.to_string(), value.fg(right_color)),
    ])
}
