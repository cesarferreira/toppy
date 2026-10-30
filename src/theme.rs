use ratatui::style::{Color, Modifier, Style};

pub const CPU_BORDER: Color = Color::Cyan;
pub const MEM_BORDER: Color = Color::Magenta;
pub const PROC_BORDER: Color = Color::White;
pub const TREE_BORDER: Color = Color::Green;
pub const STATUS_FG: Color = Color::DarkGray;
pub const HEADER_FG: Color = Color::Yellow;
pub const SELECTED_BG: Color = Color::Rgb(32, 56, 88);
pub const SELECTED_FG: Color = Color::Rgb(255, 255, 255);

pub const STATUS_KEY_HELP: Color = Color::Rgb(120, 200, 255);
pub const STATUS_KEY_NAV: Color = Color::Rgb(130, 220, 180);
pub const STATUS_KEY_SORT: Color = Color::Rgb(255, 210, 120);
pub const STATUS_KEY_FILTER: Color = Color::Rgb(190, 150, 255);
pub const STATUS_KEY_TREE: Color = Color::Rgb(120, 230, 170);
pub const STATUS_KEY_KILL: Color = Color::Rgb(255, 120, 130);
pub const STATUS_KEY_QUIT: Color = Color::Rgb(255, 140, 140);
pub const STATUS_LABEL: Color = Color::Rgb(150, 165, 190);
pub const STATUS_ACTION: Color = Color::Rgb(95, 105, 125);
pub const STATUS_MESSAGE: Color = Color::Rgb(255, 220, 120);

pub const BAR_BRACKET: Color = Color::Rgb(90, 96, 120);
pub const BAR_TRACK: Color = Color::Rgb(48, 52, 68);
pub const BAR_LABEL: Color = Color::Rgb(170, 210, 255);
pub const BAR_TOTAL: Color = Color::Rgb(130, 145, 175);
pub const BAR_SEP: Color = Color::Rgb(90, 100, 125);
pub const BAR_CACHE: Color = Color::Rgb(120, 190, 255);
pub const BAR_SWAP: Color = Color::Rgb(255, 120, 150);

// Process list palette, from btop's default theme.
const PROC_FG: Color = Color::Rgb(0xcc, 0xcc, 0xcc);
const PROC_LOW: (u8, u8, u8) = (0x80, 0xd0, 0xa3);
const PROC_MID: (u8, u8, u8) = (0xdc, 0xd1, 0x79);
const PROC_HIGH: (u8, u8, u8) = (0xd4, 0x54, 0x54);

/// Foreground colors for one process row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowColors {
    /// PID and command text.
    pub base: Color,
    /// CPU% and program name.
    pub cpu: Color,
    pub mem: Color,
}

/// btop-style row colors: CPU and memory pick a green→yellow→red color by
/// usage, independent of row position or selection.
pub fn row_colors(cpu_pct: f32, mem_pct: f32) -> RowColors {
    RowColors {
        base: PROC_FG,
        cpu: usage_gradient(cpu_pct),
        mem: usage_gradient(mem_pct),
    }
}

fn usage_gradient(pct: f32) -> Color {
    let pct = pct.clamp(0.0, 100.0);
    if pct < 50.0 {
        lerp(PROC_LOW, PROC_MID, pct / 50.0)
    } else {
        lerp(PROC_MID, PROC_HIGH, (pct - 50.0) / 50.0)
    }
}

fn lerp(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color::Rgb(mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

pub fn utilization_color(pct: f32) -> Color {
    if pct < 40.0 {
        Color::Rgb(80, 210, 130)
    } else if pct < 70.0 {
        Color::Rgb(240, 190, 70)
    } else {
        Color::Rgb(255, 95, 95)
    }
}

pub fn utilization_style(pct: f32) -> Style {
    Style::default()
        .fg(utilization_color(pct))
        .add_modifier(Modifier::BOLD)
}

pub fn header_style() -> Style {
    Style::default().fg(HEADER_FG).add_modifier(Modifier::BOLD)
}

pub fn dim_style() -> Style {
    Style::default().fg(STATUS_FG)
}

pub fn meter_label_style() -> Style {
    Style::default().fg(BAR_LABEL).add_modifier(Modifier::BOLD)
}

pub fn meter_total_style() -> Style {
    Style::default().fg(BAR_TOTAL).add_modifier(Modifier::BOLD)
}

pub fn meter_sep_style() -> Style {
    Style::default().fg(BAR_SEP)
}

pub fn status_label_style() -> Style {
    Style::default().fg(STATUS_LABEL)
}

pub fn status_action_style() -> Style {
    Style::default().fg(STATUS_ACTION)
}

pub fn status_message_style() -> Style {
    Style::default()
        .fg(STATUS_MESSAGE)
        .add_modifier(Modifier::BOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_colors_follow_usage_levels() {
        let idle = row_colors(0.0, 0.0);
        assert_eq!(idle.base, Color::Rgb(0xcc, 0xcc, 0xcc));
        assert_eq!(idle.cpu, Color::Rgb(0x80, 0xd0, 0xa3));

        let busy = row_colors(250.0, 50.0);
        assert_eq!(busy.cpu, Color::Rgb(0xd4, 0x54, 0x54));
        assert_eq!(busy.mem, Color::Rgb(0xdc, 0xd1, 0x79));
    }

    #[test]
    fn row_colors_clamp_usage_to_palette_endpoints() {
        let colors = row_colors(-5.0, 250.0);
        assert_eq!(colors.cpu, Color::Rgb(0x80, 0xd0, 0xa3));
        assert_eq!(colors.mem, Color::Rgb(0xd4, 0x54, 0x54));
    }
}
