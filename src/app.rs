use std::collections::HashSet;
use std::time::{Duration, Instant};

use sysinfo::Signal;

use crate::{
    config::Config,
    metrics::collector::{Collector, CpuSnapshot, MemorySnapshot, MetricsSnapshot},
    process::{
        model::{ProcessIdentity, ProcessRow, SortColumn, filter_indices, sort_processes},
        tree::ProcessTree,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Table,
    Tree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    Help,
    KillMenu(ProcessIdentity),
    FilterInput,
}

pub struct App {
    pub processes: Vec<ProcessRow>,
    pub filtered_indices: Vec<usize>,
    pub tree: ProcessTree,
    pub tree_expanded: HashSet<u32>,
    pub flat_tree: Vec<(u32, usize, bool)>,
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub warmed_up: bool,
    pub sort: SortColumn,
    pub sort_desc: bool,
    pub selected: usize,
    pub followed_process: Option<ProcessIdentity>,
    pub scroll_offset: usize,
    pub filter: String,
    pub mode: ViewMode,
    pub overlay: Option<Overlay>,
    pub refresh_ms: u64,
    pub visible_rows: u16,
    pub cpu_cores_expanded: bool,
    pub show_full_command: bool,
    status_message: Option<String>,
    status_message_until: Option<Instant>,
    pub should_quit: bool,
    collector: Collector,
}

impl App {
    pub fn new(refresh_ms: u64) -> Self {
        let settings = Config::load();
        let mut collector = Collector::new();
        collector.warmup();
        let snapshot = collector.refresh();

        let mut app = Self {
            processes: snapshot.processes,
            filtered_indices: Vec::new(),
            tree: ProcessTree::build(&[]),
            tree_expanded: HashSet::new(),
            flat_tree: Vec::new(),
            cpu: snapshot.cpu,
            memory: snapshot.memory,
            warmed_up: snapshot.warmed_up,
            sort: SortColumn::Cpu,
            sort_desc: true,
            selected: 0,
            followed_process: None,
            scroll_offset: 0,
            filter: String::new(),
            mode: ViewMode::Table,
            overlay: None,
            refresh_ms,
            visible_rows: 10,
            cpu_cores_expanded: settings.cpu_cores_expanded,
            show_full_command: settings.show_full_command,
            status_message: None,
            status_message_until: None,
            should_quit: false,
            collector,
        };
        app.rebuild_view();
        app
    }

    pub fn tick(&mut self) {
        let snapshot = self.collector.refresh();
        self.apply_snapshot(snapshot);
        if self
            .status_message_until
            .is_some_and(|until| Instant::now() >= until)
        {
            self.status_message = None;
            self.status_message_until = None;
        }
    }

    fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_message_until = Some(Instant::now() + Duration::from_secs(3));
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message_until
            .filter(|&until| Instant::now() < until)?;
        self.status_message.as_deref()
    }

    pub fn force_refresh(&mut self) {
        self.tick();
        self.set_status_message("refreshed");
    }

    pub fn slower_refresh(&mut self) {
        self.refresh_ms = (self.refresh_ms + 500).min(crate::input::MAX_REFRESH_MS);
        self.set_status_message(format!("refresh {}ms", self.refresh_ms));
    }

    pub fn faster_refresh(&mut self) {
        self.refresh_ms = self
            .refresh_ms
            .saturating_sub(500)
            .max(crate::input::MIN_REFRESH_MS);
        self.set_status_message(format!("refresh {}ms", self.refresh_ms));
    }

    fn apply_snapshot(&mut self, snapshot: MetricsSnapshot) {
        // Do not carry expansion state across PID reuse.
        if !self.tree_expanded.is_empty() {
            let expanded_identities: HashSet<_> = self
                .processes
                .iter()
                .filter(|process| self.tree_expanded.contains(&process.pid))
                .map(ProcessRow::identity)
                .collect();
            self.tree_expanded = snapshot
                .processes
                .iter()
                .filter(|process| expanded_identities.contains(&process.identity()))
                .map(|process| process.pid)
                .collect();
        }
        self.cpu = snapshot.cpu;
        self.memory = snapshot.memory;
        self.warmed_up = snapshot.warmed_up;
        self.processes = snapshot.processes;
        self.rebuild_view();
    }

    pub fn rebuild_view(&mut self) {
        sort_processes(
            &mut self.processes,
            self.sort,
            self.sort_desc,
            self.show_full_command,
        );
        self.filtered_indices = filter_indices(&self.processes, &self.filter);
        if self.mode == ViewMode::Tree {
            self.tree = ProcessTree::build(&self.processes);
            self.rebuild_flat_tree();
        } else {
            self.flat_tree.clear();
        }
        if let Some(identity) = self.followed_process {
            let position = if self.mode == ViewMode::Tree {
                self.flat_tree.iter().position(|(pid, _, _)| {
                    self.tree
                        .by_pid
                        .get(pid)
                        .is_some_and(|node| self.processes[node.index].identity() == identity)
                })
            } else {
                self.filtered_indices
                    .iter()
                    .position(|&i| self.processes[i].identity() == identity)
            };
            if let Some(position) = position {
                self.selected = position;
            } else {
                self.followed_process = None;
                self.set_status_message(format!(
                    "stopped following PID {} (no longer visible)",
                    identity.pid
                ));
            }
        }
        self.clamp_selection();
        self.ensure_visible();
    }

    fn rebuild_flat_tree(&mut self) {
        if !self.filter.is_empty() {
            let mut visible = HashSet::new();
            for &index in &self.filtered_indices {
                let mut current = Some(self.processes[index].pid);
                while let Some(pid) = current {
                    if !visible.insert(pid) {
                        break;
                    }
                    current = self
                        .tree
                        .by_pid
                        .get(&pid)
                        .and_then(|node| self.processes[node.index].parent_pid);
                }
            }
            self.flat_tree = self.tree.flatten_visible(&visible);
            self.flat_tree.retain(|(pid, _, _)| visible.contains(pid));
            return;
        }
        self.flat_tree = self.tree.flatten_visible(&self.tree_expanded);
    }

    pub fn selected_process(&self) -> Option<&ProcessRow> {
        if self.mode == ViewMode::Tree {
            let (pid, _, _) = self.flat_tree.get(self.selected)?;
            let node = self.tree.by_pid.get(pid)?;
            return Some(&self.processes[node.index]);
        }

        self.filtered_indices
            .get(self.selected)
            .map(|&i| &self.processes[i])
    }

    pub fn list_len(&self) -> usize {
        if self.mode == ViewMode::Tree {
            self.flat_tree.len()
        } else {
            self.filtered_indices.len()
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        self.stop_following();
        let len = self.list_len();
        if len == 0 {
            self.selected = 0;
            self.scroll_offset = 0;
            return;
        }

        let new = (self.selected as i32 + delta).clamp(0, len as i32 - 1) as usize;
        self.selected = new;
        self.ensure_visible();
    }

    pub fn move_page(&mut self, page_size: u16, up: bool) {
        let delta = if up {
            -(page_size as i32)
        } else {
            page_size as i32
        };
        self.move_selection(delta);
    }

    pub fn move_home(&mut self) {
        self.stop_following();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn move_end(&mut self) {
        self.stop_following();
        let len = self.list_len();
        if len == 0 {
            return;
        }
        self.selected = len - 1;
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        let visible = self.visible_rows.max(1) as usize;
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible {
            self.scroll_offset = self.selected + 1 - visible;
        }
    }

    fn clamp_selection(&mut self) {
        let len = self.list_len();
        if len == 0 {
            self.selected = 0;
            self.scroll_offset = 0;
            return;
        }
        if self.selected >= len {
            self.selected = len - 1;
        }
        if self.scroll_offset > self.selected {
            self.scroll_offset = self.selected;
        }
    }

    pub fn set_sort(&mut self, column: SortColumn) {
        if self.sort == column {
            self.sort_desc = !self.sort_desc;
        } else {
            self.sort = column;
            self.sort_desc = column != SortColumn::Pid;
        }
        self.rebuild_view();
    }

    pub fn enter_filter(&mut self) {
        self.overlay = Some(Overlay::FilterInput);
    }

    pub fn push_filter_char(&mut self, ch: char) {
        self.filter.push(ch);
        self.rebuild_view();
        self.move_home();
    }

    pub fn pop_filter_char(&mut self) {
        self.filter.pop();
        self.rebuild_view();
        self.move_home();
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.rebuild_view();
        self.move_home();
    }

    pub fn exit_overlay(&mut self) {
        self.overlay = None;
    }

    pub fn toggle_help(&mut self) {
        self.overlay = if self.overlay == Some(Overlay::Help) {
            None
        } else {
            Some(Overlay::Help)
        };
    }

    pub fn toggle_tree(&mut self) {
        self.stop_following();
        if self.mode == ViewMode::Tree {
            self.mode = ViewMode::Table;
            self.flat_tree.clear();
        } else {
            self.mode = ViewMode::Tree;
            self.selected = 0;
            self.scroll_offset = 0;
            self.tree = ProcessTree::build(&self.processes);
            self.rebuild_flat_tree();
        }
    }

    pub fn toggle_cpu_cores(&mut self) {
        self.cpu_cores_expanded = !self.cpu_cores_expanded;
        self.save_config();
        self.set_status_message(if self.cpu_cores_expanded {
            "CPU per-core view"
        } else {
            "CPU average view"
        });
    }

    pub fn toggle_full_command(&mut self) {
        self.show_full_command = !self.show_full_command;
        self.save_config();
        self.rebuild_view();
        self.set_status_message(if self.show_full_command {
            "showing full command"
        } else {
            "showing program name"
        });
    }

    fn save_config(&self) {
        let _ = Config {
            cpu_cores_expanded: self.cpu_cores_expanded,
            show_full_command: self.show_full_command,
        }
        .save();
    }

    pub fn open_kill_menu(&mut self) {
        if let Some(process) = self.selected_process() {
            self.overlay = Some(Overlay::KillMenu(process.identity()));
        }
    }

    pub fn toggle_follow(&mut self) {
        if self.followed_process.is_some() {
            self.stop_following();
        } else if let Some(process) = self.selected_process() {
            self.followed_process = Some(process.identity());
        }
    }

    fn stop_following(&mut self) {
        if self.followed_process.take().is_some() {
            self.set_status_message("follow off");
        }
    }

    pub fn kill_selected(&mut self, signal: Signal) {
        let Some(Overlay::KillMenu(target)) = self.overlay else {
            return;
        };
        let pid = target.pid;

        match crate::process::actions::kill_process(target, signal) {
            Ok(()) => {
                self.set_status_message(format!(
                    "sent {} to pid {pid}",
                    crate::process::actions::signal_label(signal)
                ));
            }
            Err(err) => {
                self.set_status_message(format!("kill failed: {err:#}"));
            }
        }
        self.exit_overlay();
    }

    pub fn toggle_tree_node(&mut self) {
        if self.mode != ViewMode::Tree {
            return;
        }
        let Some(&(pid, _, has_children)) = self.flat_tree.get(self.selected) else {
            return;
        };
        if !has_children {
            return;
        }
        if !self.tree_expanded.remove(&pid) {
            self.tree_expanded.insert(pid);
        }
        self.rebuild_flat_tree();
    }

    pub fn collapse_selected_tree_node(&mut self) {
        if self.mode != ViewMode::Tree {
            return;
        }
        let Some(&(pid, _, _)) = self.flat_tree.get(self.selected) else {
            return;
        };
        if self.tree_expanded.remove(&pid) {
            self.rebuild_flat_tree();
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
