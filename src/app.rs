use std::collections::HashSet;

use sysinfo::Signal;

use crate::{
    metrics::collector::{Collector, CpuSnapshot, MemorySnapshot, MetricsSnapshot},
    process::{
        model::{filter_indices, sort_processes, ProcessRow, SortColumn},
        tree::ProcessTree,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Table,
    Tree,
    Help,
    KillMenu,
    FilterInput,
}

pub struct App {
    pub processes: Vec<ProcessRow>,
    pub filtered_indices: Vec<usize>,
    pub tree: ProcessTree,
    pub tree_expanded: HashSet<u32>,
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub warmed_up: bool,
    pub sort: SortColumn,
    pub sort_desc: bool,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub mode: ViewMode,
    pub refresh_ms: u64,
    pub visible_rows: u16,
    pub status_message: Option<String>,
    pub should_quit: bool,
    collector: Collector,
}

impl App {
    pub fn new(refresh_ms: u64) -> Self {
        let mut collector = Collector::new();
        collector.warmup();
        let snapshot = collector.refresh();

        let mut app = Self {
            processes: snapshot.processes,
            filtered_indices: Vec::new(),
            tree: ProcessTree::build(&[]),
            tree_expanded: HashSet::new(),
            cpu: snapshot.cpu,
            memory: snapshot.memory,
            warmed_up: snapshot.warmed_up,
            sort: SortColumn::Cpu,
            sort_desc: true,
            selected: 0,
            scroll_offset: 0,
            filter: String::new(),
            mode: ViewMode::Table,
            refresh_ms,
            visible_rows: 10,
            status_message: None,
            should_quit: false,
            collector,
        };
        app.rebuild_view();
        app
    }

    pub fn tick(&mut self) {
        let snapshot = self.collector.refresh();
        self.apply_snapshot(snapshot);
        self.rebuild_view();
        self.status_message = None;
    }

    pub fn force_refresh(&mut self) {
        self.tick();
        self.status_message = Some("refreshed".to_string());
    }

    fn apply_snapshot(&mut self, snapshot: MetricsSnapshot) {
        self.cpu = snapshot.cpu;
        self.memory = snapshot.memory;
        self.warmed_up = snapshot.warmed_up;
        self.processes = snapshot.processes;
    }

    pub fn rebuild_view(&mut self) {
        sort_processes(&mut self.processes, self.sort, self.sort_desc);
        self.tree = ProcessTree::build(&self.processes);
        self.filtered_indices = filter_indices(&self.processes, &self.filter);
        self.clamp_selection();
    }

    pub fn selected_process(&self) -> Option<&ProcessRow> {
        if self.mode == ViewMode::Tree {
            let flat = self
                .tree
                .flatten_visible(&self.processes, &self.tree_expanded);
            let (pid, _, _) = flat.get(self.selected)?;
            let node = self.tree.by_pid.get(pid)?;
            return Some(&self.processes[node.index]);
        }

        self.filtered_indices
            .get(self.selected)
            .map(|&i| &self.processes[i])
    }

    pub fn selected_pid(&self) -> Option<u32> {
        self.selected_process().map(|p| p.pid)
    }

    pub fn list_len(&self) -> usize {
        if self.mode == ViewMode::Tree {
            self.tree
                .flatten_visible(&self.processes, &self.tree_expanded)
                .len()
        } else {
            self.filtered_indices.len()
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
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
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn move_end(&mut self) {
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
        self.mode = ViewMode::FilterInput;
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
        self.mode = if self.mode == ViewMode::Tree {
            ViewMode::Tree
        } else {
            ViewMode::Table
        };
    }

    pub fn toggle_help(&mut self) {
        self.mode = if self.mode == ViewMode::Help {
            ViewMode::Table
        } else {
            ViewMode::Help
        };
    }

    pub fn toggle_tree(&mut self) {
        if self.mode == ViewMode::Tree {
            self.mode = ViewMode::Table;
        } else {
            self.mode = ViewMode::Tree;
            self.selected = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn open_kill_menu(&mut self) {
        if self.selected_process().is_some() {
            self.mode = ViewMode::KillMenu;
        }
    }

    pub fn kill_selected(&mut self, signal: Signal) {
        let Some(pid) = self.selected_pid() else {
            return;
        };

        match crate::process::actions::kill_process(self.collector.system(), pid, signal) {
            Ok(()) => {
                self.status_message = Some(format!(
                    "sent {} to pid {pid}",
                    crate::process::actions::signal_label(signal)
                ));
                self.mode = ViewMode::Table;
            }
            Err(err) => {
                self.status_message = Some(format!("kill failed: {err:#}"));
                self.mode = ViewMode::Table;
            }
        }
    }

    pub fn toggle_tree_node(&mut self) {
        if self.mode != ViewMode::Tree {
            return;
        }
        let flat = self
            .tree
            .flatten_visible(&self.processes, &self.tree_expanded);
        let Some((pid, _, has_children)) = flat.get(self.selected) else {
            return;
        };
        if !has_children {
            return;
        }
        if self.tree_expanded.contains(pid) {
            self.tree_expanded.remove(pid);
        } else {
            self.tree_expanded.insert(*pid);
        }
    }

}
