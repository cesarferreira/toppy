use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

use crate::metrics::{collector::MemorySnapshot, format};
use crate::theme::MEM_BORDER;
use crate::ui::widgets::bar::BarGauge;

pub fn render_memory(frame: &mut Frame, area: Rect, mem: &MemorySnapshot) {
    let block = Block::default()
        .title(" Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MEM_BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let ram_pct = format::percent(mem.used, mem.total);
    let swap_pct = format::percent(mem.swap_used, mem.swap_total);

    let ram_suffix = format!(
        "{} ({})",
        format::format_bytes_pair(mem.used, mem.total),
        format::format_percent(ram_pct)
    );
    let swap_suffix = format!(
        "{} ({})",
        format::format_bytes_pair(mem.swap_used, mem.swap_total),
        format::format_percent(swap_pct)
    );

    frame.render_widget(
        BarGauge::new("RAM", ram_pct, ram_suffix),
        rows[0],
    );
    frame.render_widget(
        BarGauge::new("Swap", swap_pct, swap_suffix),
        rows[1],
    );
}
