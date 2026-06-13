use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub struct RootLayout {
    pub cpu: Rect,
    pub memory: Rect,
    pub main: Rect,
    pub status: Rect,
}

pub fn split_root(frame: &Frame, core_count: usize) -> RootLayout {
    let cpu_height = core_count.max(1).min(16) as u16 + 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(cpu_height),
            Constraint::Length(3),
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
