use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

pub struct RootLayout {
    pub cpu: Rect,
    pub memory: Rect,
    pub main: Rect,
    pub status: Rect,
}

pub fn split_root(frame: &Frame, core_count: usize, cpu_expanded: bool) -> RootLayout {
    let cpu_height = if cpu_expanded {
        core_count.clamp(1, 16) as u16 + 2
    } else {
        3
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(cpu_height),
            Constraint::Length(4),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(frame.area());

    RootLayout {
        cpu: chunks[0],
        memory: chunks[1],
        main: chunks[2],
        status: chunks[3],
    }
}
