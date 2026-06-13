use ratatui::style::{Color, Modifier, Style};

pub const CPU_BORDER: Color = Color::Cyan;
pub const MEM_BORDER: Color = Color::Magenta;
pub const PROC_BORDER: Color = Color::White;
pub const TREE_BORDER: Color = Color::Green;
pub const STATUS_FG: Color = Color::DarkGray;
pub const HEADER_FG: Color = Color::Yellow;
pub const SELECTED_BG: Color = Color::DarkGray;

pub fn utilization_color(pct: f32) -> Color {
    if pct < 40.0 {
        Color::Green
    } else if pct < 70.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

pub fn header_style() -> Style {
    Style::default().fg(HEADER_FG).add_modifier(Modifier::BOLD)
}

pub fn dim_style() -> Style {
    Style::default().fg(STATUS_FG)
}
