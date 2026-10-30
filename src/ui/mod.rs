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
    use ratatui::{
        Terminal,
        backend::TestBackend,
        buffer::{Buffer, CellWidth},
        layout::Rect,
        style::Color,
    };

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

    fn selected_process_panel(
        mode: ViewMode,
        width: u16,
        full_command: bool,
        scroll_offset: usize,
    ) -> (Buffer, Rect) {
        let mut app = fixture();
        for process in &mut app.processes {
            process.cmd = "/usr/bin/界worker --jobs".into();
            process.cmd_lower = process.cmd.to_lowercase();
            process.name = "界worker".into();
            process.name_lower = process.name.to_lowercase();
        }
        if mode == ViewMode::Tree {
            app.toggle_tree();
            app.toggle_tree_node();
        }
        app.show_full_command = full_command;
        app.selected = scroll_offset;
        app.scroll_offset = scroll_offset;
        let panel = Rect::new(2, 1, width - 4, 12);
        let mut terminal = Terminal::new(TestBackend::new(width, 16)).unwrap();
        terminal
            .draw(|frame| match mode {
                ViewMode::Table => processes::render_processes(frame, panel, &app),
                ViewMode::Tree => tree::render_tree(frame, panel, &app),
            })
            .unwrap();
        let selected = Rect::new(
            panel.x + 1,
            panel.y + if mode == ViewMode::Table { 2 } else { 1 },
            panel.width - 2,
            1,
        );
        (terminal.backend().buffer().clone(), selected)
    }

    #[test]
    fn selected_row_background_fills_panel_width() {
        for mode in [ViewMode::Table, ViewMode::Tree] {
            for width in [36, 80, 120] {
                for full_command in [false, true] {
                    for scroll_offset in [0, 1] {
                        let (buffer, row) =
                            selected_process_panel(mode, width, full_command, scroll_offset);
                        let background = buffer[(row.x, row.y)].bg;
                        assert_ne!(background, Color::Reset);
                        let mut x = row.x;
                        while x < row.right() {
                            let cell = &buffer[(x, row.y)];
                            assert_eq!(cell.bg, background, "{mode:?} at x={x}");
                            // A wide glyph paints its continuation cell; TestBackend
                            // intentionally leaves that hidden cell at its default style.
                            x += cell.cell_width().max(1);
                        }
                        // Highlight stops at the borders and does not color other rows.
                        for x in [row.x - 1, row.right()] {
                            assert_eq!(buffer[(x, row.y)].bg, Color::Reset);
                        }
                        assert_eq!(buffer[(row.x, row.y + 1)].bg, Color::Reset);
                    }
                }
            }
        }
    }

    #[test]
    fn selected_row_text_is_high_contrast_in_both_views() {
        for mode in [ViewMode::Table, ViewMode::Tree] {
            for full_command in [false, true] {
                let (buffer, row) = selected_process_panel(mode, 80, full_command, 0);
                for x in row.x..row.right() {
                    let cell = &buffer[(x, row.y)];
                    if !cell.symbol().trim().is_empty() {
                        assert_eq!(cell.fg, Color::Rgb(255, 255, 255), "{mode:?} at x={x}");
                        assert_eq!(cell.bg, Color::Rgb(32, 56, 88), "{mode:?} at x={x}");
                    }
                }
                // Unselected rows retain their usage-dependent foreground colors.
                let foregrounds: std::collections::HashSet<_> = (row.x..row.right())
                    .filter_map(|x| {
                        let cell = &buffer[(x, row.y + 1)];
                        (!cell.symbol().trim().is_empty()).then_some(cell.fg)
                    })
                    .collect();
                assert!(foregrounds.len() > 1);
            }
        }
    }

    #[test]
    fn unselected_row_colors_do_not_fade_with_distance_from_selection() {
        for mode in [ViewMode::Table, ViewMode::Tree] {
            for selected in [0, 2] {
                let mut app = fixture();
                for process in &mut app.processes {
                    process.cpu = 50.0;
                    process.parent_pid = None;
                }
                app.rebuild_view();
                if mode == ViewMode::Tree {
                    app.toggle_tree();
                }
                app.selected = selected;
                let mut terminal = Terminal::new(TestBackend::new(80, 12)).unwrap();
                terminal
                    .draw(|frame| match mode {
                        ViewMode::Table => processes::render_processes(frame, frame.area(), &app),
                        ViewMode::Tree => tree::render_tree(frame, frame.area(), &app),
                    })
                    .unwrap();
                let buffer = terminal.backend().buffer();
                let first_row = if mode == ViewMode::Table { 2 } else { 1 };
                for index in 0..3 {
                    if index == selected {
                        continue;
                    }
                    let y = first_row + index as u16;
                    let text: String = (0..80).map(|x| buffer[(x, y)].symbol()).collect();
                    let pid_x = text.find(char::is_numeric).unwrap() as u16;
                    assert_eq!(buffer[(pid_x, y)].fg, Color::Rgb(204, 204, 204));
                    for label in ["50.0%", "4.0 KB", "worker-"] {
                        let x = text.find(label).unwrap() as u16;
                        assert_eq!(
                            buffer[(x, y)].fg,
                            Color::Rgb(220, 209, 121),
                            "{mode:?} row={index} {label}"
                        );
                    }
                }
            }
        }
    }
}
