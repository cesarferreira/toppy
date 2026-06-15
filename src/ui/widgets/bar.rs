use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::{
    metrics::format::format_percent,
    theme::{self, utilization_color},
};

const EIGHTHS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

#[derive(Clone, Copy)]
pub struct BarSegment {
    pub fraction: f32,
    pub color: Color,
}

pub struct BytePairSuffix {
    pub used: String,
    pub total: String,
    pub accent: Color,
}

pub struct MeterBar {
    pub label: String,
    pub label_width: usize,
    pub pct: f32,
    pub segments: Vec<BarSegment>,
    pub pct_inside: bool,
    pub suffix: Option<BytePairSuffix>,
    /// Fixed width for the suffix column so sibling bars align.
    pub suffix_slot_width: usize,
}

impl MeterBar {
    pub fn utilization(label: impl Into<String>, pct: f32) -> Self {
        let pct = pct.clamp(0.0, 100.0);
        Self {
            label: label.into(),
            label_width: 4,
            pct,
            segments: vec![BarSegment {
                fraction: pct / 100.0,
                color: utilization_color(pct),
            }],
            pct_inside: true,
            suffix: None,
            suffix_slot_width: 0,
        }
    }

    pub fn with_label_width(mut self, width: usize) -> Self {
        self.label_width = width;
        self
    }

    pub fn with_pct_inside(mut self, inside: bool) -> Self {
        self.pct_inside = inside;
        self
    }

    pub fn with_byte_pair_suffix(
        mut self,
        used: impl Into<String>,
        total: impl Into<String>,
        accent: Color,
        slot_width: usize,
    ) -> Self {
        self.suffix = Some(BytePairSuffix {
            used: used.into(),
            total: total.into(),
            accent,
        });
        self.suffix_slot_width = slot_width;
        self
    }

    pub fn with_segments(mut self, segments: Vec<BarSegment>) -> Self {
        if !segments.is_empty() {
            self.segments = segments;
        }
        self
    }
}

impl Widget for MeterBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height == 0 {
            return;
        }

        let area_w = area.width as usize;
        let pct_text = format_percent(self.pct);
        let suffix_slot = self.suffix_slot_width;
        let label_width = self.label_width;

        // label + space + "[" + inner + "]" + suffix slot
        let overhead = label_width + 1 + 2 + suffix_slot;
        if overhead >= area_w {
            return;
        }

        let budget = area_w - overhead;
        let pct_inside = self.pct_inside && budget > pct_text.len() + 1;
        let inner_width = budget.max(2);

        let y = area.y;
        let mut x = area.x;

        // Right-aligned label.
        let label_style = theme::meter_label_style();
        let label_chars = self.label.chars().count();
        let label_pad = label_width.saturating_sub(label_chars);
        for _ in 0..label_pad {
            buf[(x, y)].set_char(' ').set_style(label_style);
            x += 1;
        }
        for ch in self.label.chars().take(label_width) {
            buf[(x, y)].set_char(ch).set_style(label_style);
            x += 1;
        }

        // Space separator.
        buf[(x, y)].set_char(' ');
        x += 1;

        // Opening bracket.
        let bracket_style = Style::default().fg(theme::BAR_BRACKET);
        buf[(x, y)].set_char('[').set_style(bracket_style);
        x += 1;

        let (track_width, pct_overlay) = if pct_inside {
            (inner_width.saturating_sub(pct_text.len()).max(1), true)
        } else {
            (inner_width, false)
        };

        render_fill_into(buf, x, y, track_width, &self.segments);
        x += track_width as u16;

        if pct_overlay {
            let pct_style = theme::utilization_style(self.pct);
            for ch in pct_text.chars() {
                buf[(x, y)].set_char(ch).set_style(pct_style);
                x += 1;
            }
        }

        // Closing bracket.
        buf[(x, y)].set_char(']').set_style(bracket_style);
        x += 1;

        render_suffix_into(buf, x, y, self.suffix, suffix_slot);
    }
}

fn render_suffix_into(
    buf: &mut Buffer,
    mut x: u16,
    y: u16,
    suffix: Option<BytePairSuffix>,
    slot_width: usize,
) {
    if slot_width == 0 {
        return;
    }

    let Some(BytePairSuffix { used, total, accent }) = suffix else {
        for _ in 0..slot_width {
            buf[(x, y)].set_char(' ');
            x += 1;
        }
        return;
    };

    let bracket_style = Style::default().fg(theme::BAR_BRACKET);
    let used_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
    let sep_style = theme::meter_sep_style();
    let total_style = theme::meter_total_style();

    let inner_width = slot_width.saturating_sub(3);
    let body_len = used.len() + 1 + total.len();
    let pad = inner_width.saturating_sub(body_len);

    buf[(x, y)].set_char(' ');
    x += 1;
    buf[(x, y)].set_char('[').set_style(bracket_style);
    x += 1;
    for _ in 0..pad {
        buf[(x, y)].set_char(' ');
        x += 1;
    }
    for ch in used.chars() {
        buf[(x, y)].set_char(ch).set_style(used_style);
        x += 1;
    }
    buf[(x, y)].set_char('/').set_style(sep_style);
    x += 1;
    for ch in total.chars() {
        buf[(x, y)].set_char(ch).set_style(total_style);
        x += 1;
    }
    buf[(x, y)].set_char(']').set_style(bracket_style);
}

fn render_fill_into(buf: &mut Buffer, base_x: u16, y: u16, width: usize, segments: &[BarSegment]) {
    if width == 0 {
        return;
    }

    let track_style = Style::default().fg(theme::BAR_TRACK);
    for i in 0..width {
        buf[(base_x + i as u16, y)]
            .set_char('·')
            .set_style(track_style);
    }

    let mut cursor = 0.0_f32;
    for segment in segments {
        let seg_width = (segment.fraction.clamp(0.0, 1.0) * width as f32).max(0.0);
        let end = (cursor + seg_width).min(width as f32);
        let start_idx = cursor.floor() as usize;
        let end_idx = end.ceil() as usize;
        let style = Style::default().fg(segment.color);

        for idx in start_idx..end_idx.min(width) {
            let cell_start = idx as f32;
            let cell_end = cell_start + 1.0;
            let overlap_start = cursor.max(cell_start);
            let overlap_end = end.min(cell_end);
            let fill = (overlap_end - overlap_start).clamp(0.0, 1.0);
            if fill > 0.0 {
                let level = (fill * 8.0).round() as usize;
                let ch = EIGHTHS[level.min(8)];
                buf[(base_x + idx as u16, y)].set_char(ch).set_style(style);
            }
        }

        cursor = end;
    }
}

pub fn byte_pair_slot_width(pairs: &[(&str, &str)]) -> usize {
    pairs
        .iter()
        .map(|(used, total)| used.len() + 1 + total.len() + 3) // " [" + content + "]"
        .max()
        .unwrap_or(0)
}
