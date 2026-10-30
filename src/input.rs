use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{App, Overlay, ViewMode};
use crate::process::model::SortColumn;
use sysinfo::Signal;

pub const MIN_REFRESH_MS: u64 = 200;
pub const MAX_REFRESH_MS: u64 = 10_000;
pub const DEFAULT_REFRESH_MS: u64 = 1500;

pub fn handle_event(app: &mut App, event: Event) -> bool {
    match event {
        Event::Key(key) if key.kind != KeyEventKind::Release => handle_key(app, key),
        Event::Resize(_, _) => true,
        _ => false,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return true;
    }

    match app.overlay {
        Some(Overlay::FilterInput) => {
            handle_filter_input(app, key);
            return true;
        }
        Some(Overlay::KillMenu(_)) => {
            handle_kill_menu(app, key);
            return true;
        }
        Some(Overlay::Help) => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1)) {
                app.exit_overlay();
            }
            return true;
        }
        None => {}
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') | KeyCode::F(1) => app.toggle_help(),
        KeyCode::Char('t') => app.toggle_tree(),
        KeyCode::Char('c') => app.toggle_cpu_cores(),
        KeyCode::Char('p') => app.toggle_full_command(),
        KeyCode::Char('r') => app.force_refresh(),
        KeyCode::Char('+') | KeyCode::Char('=') => app.slower_refresh(),
        KeyCode::Char('-') | KeyCode::Char('_') => app.faster_refresh(),
        KeyCode::Char('k') | KeyCode::F(9) => app.open_kill_menu(),
        KeyCode::Char('P') => app.set_sort(SortColumn::Pid),
        KeyCode::Char('C') => app.set_sort(SortColumn::Cpu),
        KeyCode::Char('M') => app.set_sort(SortColumn::Mem),
        KeyCode::Char('T') => app.set_sort(SortColumn::Command),
        KeyCode::Char('/') => app.enter_filter(),
        KeyCode::Up => app.move_selection(-1),
        KeyCode::Down => app.move_selection(1),
        KeyCode::PageUp => app.move_page(app.visible_rows, true),
        KeyCode::PageDown => app.move_page(app.visible_rows, false),
        KeyCode::Home => app.move_home(),
        KeyCode::End => app.move_end(),
        KeyCode::Right | KeyCode::Enter if app.mode == ViewMode::Tree => app.toggle_tree_node(),
        KeyCode::Left if app.mode == ViewMode::Tree => app.collapse_selected_tree_node(),
        KeyCode::Esc => {
            app.clear_filter();
            app.exit_overlay();
        }
        _ => {}
    }

    true
}

fn handle_filter_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.clear_filter();
            app.exit_overlay();
        }
        KeyCode::Enter => {
            app.exit_overlay();
        }
        KeyCode::Backspace => app.pop_filter_char(),
        KeyCode::Char(c) => app.push_filter_char(c),
        _ => {}
    }
}

fn handle_kill_menu(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.exit_overlay(),
        KeyCode::Char('1') => app.kill_selected(Signal::Term),
        KeyCode::Char('2') => app.kill_selected(Signal::Kill),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::fixture;
    use crossterm::event::KeyEventKind;

    fn press(app: &mut App, code: KeyCode) {
        handle_event(app, Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
    }

    #[test]
    fn f1_toggles_help_without_losing_tree_selection() {
        let mut app = fixture();
        press(&mut app, KeyCode::Char('t'));
        press(&mut app, KeyCode::F(1));
        assert_eq!(app.overlay, Some(Overlay::Help));
        assert_eq!(app.mode, ViewMode::Tree);
        press(&mut app, KeyCode::F(1));
        assert_eq!(app.overlay, None);
        assert_eq!(app.selected_process().unwrap().pid, 10);
    }

    #[test]
    fn f9_cancel_preserves_tree() {
        let mut app = fixture();
        app.toggle_tree();
        press(&mut app, KeyCode::F(9));
        assert!(matches!(app.overlay, Some(Overlay::KillMenu(_))));
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.overlay, None);
        assert_eq!(app.mode, ViewMode::Tree);
    }

    #[test]
    fn key_release_does_not_repeat_actions() {
        let mut app = fixture();
        let mut key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        handle_event(&mut app, Event::Key(key));
        assert_eq!(app.mode, ViewMode::Table);
    }

    #[test]
    fn page_down_uses_visible_row_count() {
        let mut app = fixture();
        app.visible_rows = 1;
        press(&mut app, KeyCode::PageDown);
        assert_eq!(app.selected, 1);
        assert_eq!(app.scroll_offset, 1);
    }

    #[test]
    fn filter_escape_clears_filter_and_restores_tree() {
        let mut app = fixture();
        app.toggle_tree();
        press(&mut app, KeyCode::Char('/'));
        press(&mut app, KeyCode::Char('x'));
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.filter, "");
        assert_eq!(app.mode, ViewMode::Tree);
        assert_eq!(app.overlay, None);
    }
}
