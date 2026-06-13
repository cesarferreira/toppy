use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::metrics::collector::CpuSnapshot;
use crate::theme::{self, CPU_BORDER};
use crate::ui::widgets::bar::MeterBar;

pub fn render_cpu(frame: &mut Frame, area: Rect, cpu: &CpuSnapshot, warmed_up: bool, expanded: bool) {
    let title = if expanded {
        " CPU — per core "
    } else {
        " CPU "
    };

    let block = Block::default()
        .title(title)
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

    if expanded {
        render_per_core(frame, inner, cpu);
    } else {
        let row = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        };
        let meter = MeterBar::utilization("Avg", cpu.global)
            .with_label_width(4)
            .with_pct_inside(true);
        frame.render_widget(meter, row);
    }
}

fn render_per_core(frame: &mut Frame, inner: Rect, cpu: &CpuSnapshot) {
    let cores = cpu.per_core.len().max(1);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); inner.height as usize])
        .split(inner);

    for i in 0..rows.len().min(cores) {
        let pct = cpu.per_core.get(i).copied().unwrap_or(0.0);
        let meter = MeterBar::utilization(format!("{}", i + 1), pct).with_label_width(2);
        frame.render_widget(meter, rows[i]);
    }
}
