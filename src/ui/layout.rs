use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

pub struct RootLayout {
    pub cpu: Rect,
    pub memory: Rect,
    pub io: Rect,
    pub main: Rect,
    pub status: Rect,
}

pub fn split_root(
    frame: &Frame,
    core_count: usize,
    cpu_expanded: bool,
    show_io: bool,
) -> RootLayout {
    let cpu_height = if cpu_expanded {
        core_count.clamp(1, 16) as u16 + 2
    } else {
        3
    };
    // Two labeled rows plus the border, same height as the memory panel.
    let io_height = if show_io { 4 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(cpu_height),
            Constraint::Length(4),
            Constraint::Length(io_height),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(frame.area());

    RootLayout {
        cpu: chunks[0],
        memory: chunks[1],
        io: chunks[2],
        main: chunks[3],
        status: chunks[4],
    }
}
