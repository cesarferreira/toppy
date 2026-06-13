use ratatui::style::{Color, Modifier, Style};

pub const CPU_BORDER: Color = Color::Cyan;
pub const MEM_BORDER: Color = Color::Magenta;
pub const PROC_BORDER: Color = Color::White;
pub const TREE_BORDER: Color = Color::Green;
pub const STATUS_FG: Color = Color::DarkGray;
pub const HEADER_FG: Color = Color::Yellow;
pub const SELECTED_BG: Color = Color::DarkGray;

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
