use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::metrics::collector::CpuSnapshot;
use crate::metrics::format::format_percent;
use crate::theme::{self, CPU_BORDER};
use crate::ui::widgets::bar::MeterBar;

pub fn render_cpu(frame: &mut Frame, area: Rect, cpu: &CpuSnapshot, warmed_up: bool) {
    let block = Block::default()
        .title(" CPU ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CPU_BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    if !warmed_up {
        let msg = Paragraph::new("warming up CPU metrics...")
            .style(theme::dim_style());
        frame.render_widget(msg, inner);
        return;
    }

    let cores = cpu.per_core.len().max(1);
    let summary_rows = if inner.height > 1 { 1 } else { 0 };
    let core_rows = inner.height.saturating_sub(summary_rows).min(cores as u16);

    for i in 0..core_rows {
        let pct = cpu.per_core.get(i as usize).copied().unwrap_or(0.0);
        let row_area = Rect {
            y: inner.y + i as u16,
            height: 1,
            width: inner.width,
            x: inner.x,
        };
        let meter = MeterBar::utilization(format!("{}", i + 1), pct).with_label_width(2);
        frame.render_widget(meter, row_area);
    }

    if summary_rows == 1 {
        let summary_area = Rect {
            y: inner.y + core_rows,
            height: 1,
            width: inner.width,
            x: inner.x,
        };
        let summary = format!(
            "Avg {:>5}   cores {}",
            format_percent(cpu.global),
            cores
        );
        frame.render_widget(
            Paragraph::new(summary).style(theme::dim_style()),
            summary_area,
        );
    }
}
