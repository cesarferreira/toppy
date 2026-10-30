pub mod cpu;
pub mod help;
pub mod layout;
pub mod memory;
pub mod processes;
pub mod row;
pub mod status;
pub mod text;
pub mod tree;
pub mod widgets;

use ratatui::Frame;

use crate::app::{App, Overlay, ViewMode};

pub fn render(frame: &mut Frame, app: &mut App) {
    let root = layout::split_root(frame, app.cpu.per_core.len(), app.cpu_cores_expanded);
    app.visible_rows = root
        .main
        .height
        .saturating_sub(if app.mode == ViewMode::Table { 3 } else { 2 });

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
        ViewMode::Table => processes::render_processes(frame, root.main, app),
    }

    status::render_status(frame, root.status, app);

    match app.overlay {
        Some(Overlay::Help) => help::render_help(frame, frame.area()),
        Some(Overlay::KillMenu(target)) => help::render_kill_menu(frame, frame.area(), target.pid),
        Some(Overlay::FilterInput) => help::render_filter_input(frame, frame.area(), &app.filter),
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::fixture;
    use ratatui::{Terminal, backend::TestBackend};

    fn screen(terminal: &Terminal<TestBackend>) -> String {
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn help_shows_general_shortcuts_at_normal_size() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal
            .draw(|frame| help::render_help(frame, frame.area()))
            .unwrap();
        let text = screen(&terminal);
        assert!(text.contains("quit"));
        assert!(text.contains("F1"));
    }

    #[test]
    fn help_explains_follow_at_standard_sizes() {
        for (width, height) in [(80, 24), (120, 40)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal
                .draw(|frame| help::render_help(frame, frame.area()))
                .unwrap();
            let text = screen(&terminal);
            assert!(text.contains("follow selected process"));
            assert!(text.contains("quit"));
        }
    }

    #[test]
    fn kill_dialog_displays_pinned_pid_after_selection_moves() {
        let mut app = fixture();
        app.open_kill_menu();
        app.move_end();
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| render(frame, &mut app)).unwrap();
        assert!(screen(&terminal).contains("Kill PID 20"));
    }

    #[test]
    fn tiny_terminals_do_not_panic() {
        let mut app = fixture();
        for (width, height) in [(1, 1), (20, 5), (80, 24)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            for overlay in [None, Some(Overlay::Help), Some(Overlay::FilterInput)] {
                app.overlay = overlay;
                terminal.draw(|frame| render(frame, &mut app)).unwrap();
            }
        }
    }

    #[test]
    fn table_page_size_excludes_header_row() {
        let mut app = fixture();
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| render(frame, &mut app)).unwrap();
        // 40 rows - CPU(3) - memory(4) - status(1) - borders(2) - header(1).
        assert_eq!(app.visible_rows, 29);
        app.toggle_tree();
        terminal.draw(|frame| render(frame, &mut app)).unwrap();
        assert_eq!(app.visible_rows, 30);
    }
}
