use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::metrics::collector::CpuSnapshot;
use crate::theme::{self, CPU_BORDER};
use crate::ui::widgets::bar::BarGauge;

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
    let rows = inner.height.min(cores as u16);

    for i in 0..rows {
        let pct = cpu.per_core.get(i as usize).copied().unwrap_or(0.0);
        let row_area = Rect {
            y: inner.y + i as u16,
            height: 1,
            width: inner.width,
            x: inner.x,
        };
        let gauge = BarGauge::new(format!("C{i:<2}"), pct, "");
        frame.render_widget(gauge, row_area);
    }
}
