use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

use crate::metrics::{collector::MemorySnapshot, format};
use crate::theme::{self, BAR_SWAP, MEM_BORDER};
use crate::ui::widgets::bar::{byte_pair_slot_width, BarSegment, MeterBar};

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

    let mem_used = format::format_bytes_compact(mem.used);
    let mem_total = format::format_bytes_compact(mem.total);
    let swap_used = format::format_bytes_compact(mem.swap_used);
    let swap_total = format::format_bytes_compact(mem.swap_total);

    let suffix_slot = byte_pair_slot_width(&[
        (&mem_used, &mem_total),
        (&swap_used, &swap_total),
    ]);

    let ram_segments = ram_segments(mem);
    let ram_meter = MeterBar::utilization("Mem", ram_pct)
        .with_label_width(4)
        .with_pct_inside(false)
        .with_segments(ram_segments)
        .with_byte_pair_suffix(
            mem_used,
            mem_total,
            theme::utilization_color(ram_pct),
            suffix_slot,
        );

    let swap_meter = MeterBar::utilization("Swp", swap_pct)
        .with_label_width(4)
        .with_pct_inside(false)
        .with_segments(vec![BarSegment {
            fraction: swap_pct / 100.0,
            color: if swap_pct > 50.0 {
                theme::utilization_color(swap_pct)
            } else {
                BAR_SWAP
            },
        }])
        .with_byte_pair_suffix(
            swap_used,
            swap_total,
            if swap_pct > 50.0 {
                theme::utilization_color(swap_pct)
            } else {
                BAR_SWAP
            },
            suffix_slot,
        );

    frame.render_widget(ram_meter, rows[0]);
    frame.render_widget(swap_meter, rows[1]);
}

fn ram_segments(mem: &MemorySnapshot) -> Vec<BarSegment> {
    if mem.total == 0 {
        return vec![BarSegment {
            fraction: 0.0,
            color: theme::utilization_color(0.0),
        }];
    }

    let total = mem.total as f32;
    let used_frac = mem.used as f32 / total;
    let mut segments = vec![BarSegment {
        fraction: used_frac,
        color: theme::utilization_color(used_frac * 100.0),
    }];

    let pressure_frac = (mem.total.saturating_sub(mem.available)) as f32 / total;
    let cache_frac = pressure_frac - used_frac;
    if cache_frac > 0.01 {
        segments.push(BarSegment {
            fraction: cache_frac,
            color: theme::BAR_CACHE,
        });
    }

    segments
}
