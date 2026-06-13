pub mod cpu;
pub mod help;
pub mod layout;
pub mod memory;
pub mod processes;
pub mod status;
pub mod text;
pub mod tree;
pub mod widgets;

use ratatui::Frame;

use crate::app::{App, ViewMode};

pub fn render(frame: &mut Frame, app: &mut App) {
    let root = layout::split_root(frame, app.cpu.per_core.len(), app.cpu_cores_expanded);
    app.visible_rows = root.main.height.saturating_sub(2);

    cpu::render_cpu(
        frame,
        root.cpu,
        &app.cpu,
        app.warmed_up,
        app.cpu_cores_expanded,
    );
    memory::render_memory(frame, root.memory, &app.memory);

    match app.mode {
        ViewMode::Tree => tree::render_tree(frame, root.main, app),
        _ => processes::render_processes(frame, root.main, app),
    }

    status::render_status(frame, root.status, app);

    match app.mode {
        ViewMode::Help => help::render_help(frame, frame.area()),
        ViewMode::KillMenu => {
            if let Some(proc) = app.selected_process() {
                help::render_kill_menu(frame, frame.area(), proc.pid);
            }
        }
        ViewMode::FilterInput => help::render_filter_input(frame, frame.area(), &app.filter),
        _ => {}
    }
}
