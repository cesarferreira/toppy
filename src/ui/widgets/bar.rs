use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
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
        let label = format!("{:>width$}", self.label, width = self.label_width);
        let pct_text = format_percent(self.pct);
        let suffix_slot = self.suffix_slot_width;

        // label + space + "[" + inner + "]" + suffix slot
        let overhead = label.len() + 1 + 2 + suffix_slot;
        if overhead >= area_w {
            return;
        }

        let mut budget = area_w - overhead;
        let pct_inside = self.pct_inside && budget > pct_text.len() + 3;
        if pct_inside {
            budget -= pct_text.len();
        }

        let inner_width = budget.max(2);

        let mut spans = vec![
            Span::styled(label, theme::meter_label_style()),
            Span::raw(" "),
            Span::styled("[", Style::default().fg(theme::BAR_BRACKET)),
        ];

        spans.extend(render_fill(inner_width, &self.segments));

        if pct_inside {
            let fill_cells = ((self.pct / 100.0) * inner_width as f32).ceil() as usize;
            let pad = inner_width.saturating_sub(fill_cells + pct_text.len());
            if pad > 0 {
                spans.push(Span::styled(
                    " ".repeat(pad),
                    Style::default().fg(theme::BAR_TRACK),
                ));
            }
            spans.push(Span::styled(
                pct_text,
                theme::utilization_style(self.pct),
            ));
        }

        spans.push(Span::styled("]", Style::default().fg(theme::BAR_BRACKET)));
        spans.extend(render_suffix_slot(self.suffix, suffix_slot));

        Line::from(spans).render(area, buf);
    }
}

fn render_suffix_slot(suffix: Option<BytePairSuffix>, slot_width: usize) -> Vec<Span<'static>> {
    if slot_width == 0 {
        return Vec::new();
    }

    let Some(BytePairSuffix { used, total, accent }) = suffix else {
        return vec![Span::raw(" ".repeat(slot_width))];
    };

    let inner_width = slot_width.saturating_sub(3);
    let body_len = used.len() + 1 + total.len();
    let pad = inner_width.saturating_sub(body_len);
    vec![
        Span::raw(" "),
        Span::styled("[", Style::default().fg(theme::BAR_BRACKET)),
        Span::raw(" ".repeat(pad)),
        Span::styled(used, Style::default().fg(accent).add_modifier(Modifier::BOLD)),
        Span::styled("/", theme::meter_sep_style()),
        Span::styled(total, theme::meter_total_style()),
        Span::styled("]", Style::default().fg(theme::BAR_BRACKET)),
    ]
}

fn render_fill(width: usize, segments: &[BarSegment]) -> Vec<Span<'static>> {
    if width == 0 {
        return Vec::new();
    }

    let mut cells = vec![(theme::BAR_TRACK, '·'); width];
    let mut cursor = 0.0_f32;

    for segment in segments {
        let seg_width = (segment.fraction.clamp(0.0, 1.0) * width as f32).max(0.0);
        let end = (cursor + seg_width).min(width as f32);
        let start_idx = cursor.floor() as usize;
        let end_idx = end.ceil() as usize;

        for idx in start_idx..end_idx.min(width) {
            let cell_start = idx as f32;
            let cell_end = cell_start + 1.0;
            let overlap_start = cursor.max(cell_start);
            let overlap_end = end.min(cell_end);
            let fill = (overlap_end - overlap_start).clamp(0.0, 1.0);
            if fill > 0.0 {
                let level = (fill * 8.0).round() as usize;
                cells[idx] = (segment.color, EIGHTHS[level.min(8)]);
            }
        }

        cursor = end;
    }

    cells
        .into_iter()
        .map(|(color, ch)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect()
}

pub fn byte_pair_slot_width(pairs: &[(&str, &str)]) -> usize {
    pairs
        .iter()
        .map(|(used, total)| used.len() + 1 + total.len() + 3) // " [" + content + "]"
        .max()
        .unwrap_or(0)
}
