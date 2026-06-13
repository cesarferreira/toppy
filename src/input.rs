use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, ViewMode};
use crate::process::model::SortColumn;
use sysinfo::Signal;

pub fn handle_event(app: &mut App, event: Event) -> bool {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Resize(_, _) => true,
        _ => false,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return true;
    }

    match app.mode {
        ViewMode::FilterInput => {
            handle_filter_input(app, key);
            return true;
        }
        ViewMode::KillMenu => {
            handle_kill_menu(app, key);
            return true;
        }
        ViewMode::Help => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1)) {
                app.mode = ViewMode::Table;
            }
            return true;
        }
        ViewMode::Tree | ViewMode::Table => {}
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') | KeyCode::F(1) => app.toggle_help(),
        KeyCode::Char('t') => app.toggle_tree(),
        KeyCode::Char('r') => app.force_refresh(),
        KeyCode::Char('k') | KeyCode::F(9) => app.open_kill_menu(),
        KeyCode::Char('P') => app.set_sort(SortColumn::Pid),
        KeyCode::Char('C') => app.set_sort(SortColumn::Cpu),
        KeyCode::Char('M') => app.set_sort(SortColumn::Mem),
        KeyCode::Char('T') => app.set_sort(SortColumn::Command),
        KeyCode::Char('/') => app.enter_filter(),
        KeyCode::Up => app.move_selection(-1),
        KeyCode::Down => app.move_selection(1),
        KeyCode::PageUp => app.move_page(10, true),
        KeyCode::PageDown => app.move_page(10, false),
        KeyCode::Home => app.move_home(),
        KeyCode::End => app.move_end(),
        KeyCode::Right | KeyCode::Enter if app.mode == ViewMode::Tree => app.toggle_tree_node(),
        KeyCode::Left if app.mode == ViewMode::Tree => {
            let flat = app
                .tree
                .flatten_visible(&app.processes, &app.tree_expanded);
            if let Some((pid, _, _)) = flat.get(app.selected) {
                app.tree_expanded.remove(pid);
            }
        }
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
            app.mode = ViewMode::Table;
        }
        KeyCode::Enter => {
            app.mode = ViewMode::Table;
        }
        KeyCode::Backspace => app.pop_filter_char(),
        KeyCode::Char(c) => app.push_filter_char(c),
        _ => {}
    }
}

fn handle_kill_menu(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.mode = ViewMode::Table,
        KeyCode::Char('1') => app.kill_selected(Signal::Term),
        KeyCode::Char('2') => app.kill_selected(Signal::Kill),
        _ => {}
    }
}
