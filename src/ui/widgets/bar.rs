use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::metrics::format::format_percent;
use crate::theme::utilization_color;

pub struct BarGauge {
    pub label: String,
    pub pct: f32,
    pub suffix: String,
}

impl BarGauge {
    pub fn new(label: impl Into<String>, pct: f32, suffix: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            pct: pct.clamp(0.0, 100.0),
            suffix: suffix.into(),
        }
    }
}

impl Widget for BarGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height == 0 {
            return;
        }

        let color = utilization_color(self.pct);
        let label = format!("{:<8}", self.label);
        let pct_text = format_percent(self.pct);
        let suffix = if self.suffix.is_empty() {
            String::new()
        } else {
            format!("  {}", self.suffix)
        };

        let right_len = pct_text.len() + suffix.len() + 1;
        let bar_width = area.width.saturating_sub(label.len() as u16 + right_len as u16 + 2);

        let filled = ((self.pct / 100.0) * bar_width as f32).round() as u16;
        let empty = bar_width.saturating_sub(filled);

        let bar = format!(
            "[{}{}]",
            "█".repeat(filled as usize),
            "░".repeat(empty as usize)
        );

        let line = Line::from(vec![
            Span::styled(label, Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled(bar, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(pct_text, Style::default().fg(color)),
            Span::styled(suffix, Style::default().fg(Color::DarkGray)),
        ]);

        line.render(area, buf);
    }
}
