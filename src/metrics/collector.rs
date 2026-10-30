use std::collections::HashMap;
use std::ffi::{OsStr, OsString};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::process::model::ProcessRow;

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Debug, Clone)]
pub struct CpuSnapshot {
    pub per_core: Vec<f32>,
    pub global: f32,
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub processes: Vec<ProcessRow>,
    pub warmed_up: bool,
}

#[derive(Clone)]
struct CmdEntry {
    start_time: u64,
    source: Vec<OsString>,
    cmd: String,
    cmd_lower: String,
    name: String,
    name_lower: String,
}

impl CmdEntry {
    fn new(start_time: u64, cmd_parts: &[OsString], name: &OsStr) -> Self {
        let cmd = build_cmd_string(cmd_parts, name);
        let mut cmd_lower = cmd.clone();
        cmd_lower.make_ascii_lowercase();
        let name = name.to_string_lossy().into_owned();
        let mut name_lower = name.clone();
        name_lower.make_ascii_lowercase();
        Self {
            start_time,
            source: cmd_parts.to_vec(),
            cmd,
            cmd_lower,
            name,
            name_lower,
        }
    }

    fn update(&mut self, start_time: u64, cmd_parts: &[OsString], name: &OsStr) {
        if self.start_time != start_time
            || self.source != cmd_parts
            || self.name != name.to_string_lossy()
        {
            *self = Self::new(start_time, cmd_parts, name);
        }
    }
}

pub struct Collector {
    system: System,
    tick_count: u32,
    cmd_cache: HashMap<u32, CmdEntry>,
}

impl Collector {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_memory();
        system.refresh_cpu_usage();
        refresh_processes(&mut system);
        Self {
            system,
            tick_count: 0,
            cmd_cache: HashMap::new(),
        }
    }

    pub fn warmup(&mut self) {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.system.refresh_cpu_usage();
        refresh_processes(&mut self.system);
        self.tick_count = 1;
    }

    pub fn refresh(&mut self) -> MetricsSnapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        refresh_processes(&mut self.system);
        self.tick_count += 1;

        let cpus = self.system.cpus();
        let mut per_core: Vec<f32> = Vec::with_capacity(cpus.len());
        let mut sum = 0.0_f32;
        for cpu in cpus {
            let u = cpu.cpu_usage();
            per_core.push(u);
            sum += u;
        }
        let global = if per_core.is_empty() {
            0.0
        } else {
            sum / per_core.len() as f32
        };

        let sys_procs = self.system.processes();
        let mut processes: Vec<ProcessRow> = Vec::with_capacity(sys_procs.len());

        for (pid, process) in sys_procs {
            let pid_u32 = pid.as_u32();

            let cmd_entry = self.cmd_cache.entry(pid_u32).or_insert_with(|| {
                CmdEntry::new(process.start_time(), process.cmd(), process.name())
            });
            cmd_entry.update(process.start_time(), process.cmd(), process.name());

            processes.push(ProcessRow {
                pid: pid_u32,
                start_time: process.start_time(),
                cpu: process.cpu_usage(),
                mem_bytes: process.memory(),
                cmd: cmd_entry.cmd.clone(),
                cmd_lower: cmd_entry.cmd_lower.clone(),
                name: cmd_entry.name.clone(),
                name_lower: cmd_entry.name_lower.clone(),
                parent_pid: process.parent().map(|p| p.as_u32()),
            });
        }

        self.cmd_cache.retain(|pid, entry| {
            sys_procs
                .get(&sysinfo::Pid::from_u32(*pid))
                .is_some_and(|process| process.start_time() == entry.start_time)
        });

        MetricsSnapshot {
            cpu: CpuSnapshot { per_core, global },
            memory: MemorySnapshot {
                total: self.system.total_memory(),
                used: self.system.used_memory(),
                available: self.system.available_memory(),
                swap_total: self.system.total_swap(),
                swap_used: self.system.used_swap(),
            },
            processes,
            warmed_up: self.tick_count >= 2,
        }
    }
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

/// Same as `System::refresh_processes`, plus the command line (not loaded by
/// default), minus tasks: on Linux sysinfo otherwise lists every thread as its
/// own process (and the parent already sums them).
fn refresh_processes(system: &mut System) {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_memory()
            .with_cpu()
            .with_disk_usage()
            .with_exe(UpdateKind::OnlyIfNotSet)
            .with_cmd(UpdateKind::Always)
            .without_tasks(),
    );
}

fn build_cmd_string(cmd_parts: &[std::ffi::OsString], name: &OsStr) -> String {
    if cmd_parts.is_empty() {
        return name.to_string_lossy().into_owned();
    }
    // Pre-size the buffer to avoid reallocations during push.
    let mut needed = 0usize;
    for part in cmd_parts {
        needed += part.len() + 1;
    }
    let mut out = String::with_capacity(needed);
    let mut first = true;
    for part in cmd_parts {
        if !first {
            out.push(' ');
        }
        // OsStr::to_string_lossy borrows when valid UTF-8; only allocates on bad bytes.
        out.push_str(&part.to_string_lossy());
        first = false;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_does_not_reuse_stale_command_for_live_pid() {
        let mut collector = Collector::new();
        let pid = std::process::id();
        collector.cmd_cache.insert(
            pid,
            CmdEntry::new(0, &[], OsStr::new("previous-executable")),
        );
        let snapshot = collector.refresh();
        let process = snapshot.processes.iter().find(|p| p.pid == pid).unwrap();
        assert_ne!(process.cmd, "previous-executable");
        assert_ne!(process.name, "previous-executable");
    }

    #[test]
    fn empty_command_falls_back_to_name() {
        assert_eq!(
            build_cmd_string(&[], OsStr::new("kernel-task")),
            "kernel-task"
        );
    }

    #[test]
    fn command_preserves_arguments_and_unicode() {
        let parts = ["/bin/worker", "hello world", "東京"].map(Into::into);
        assert_eq!(
            build_cmd_string(&parts, OsStr::new("worker")),
            "/bin/worker hello world 東京"
        );
    }

    #[test]
    fn cache_updates_arguments_without_a_pid_change() {
        let mut entry = CmdEntry::new(
            100,
            &["worker".into(), "--old".into()],
            OsStr::new("worker"),
        );
        entry.update(
            100,
            &["worker".into(), "--NEW".into()],
            OsStr::new("worker"),
        );
        assert_eq!(entry.cmd, "worker --NEW");
        assert_eq!(entry.cmd_lower, "worker --new");
    }

    #[test]
    fn cache_tracks_new_identity_even_when_command_is_unchanged() {
        let mut entry = CmdEntry::new(100, &[], OsStr::new("worker"));
        entry.update(200, &[], OsStr::new("worker"));
        assert_eq!(entry.start_time, 200);
    }

    #[test]
    fn cache_retries_empty_command_and_updates_fallback_name() {
        let mut entry = CmdEntry::new(100, &[], OsStr::new("old"));
        entry.update(100, &[], OsStr::new("NEW"));
        assert_eq!(entry.cmd, "NEW");
        assert_eq!(entry.name_lower, "new");
        entry.update(
            100,
            &["/bin/new".into(), "--ready".into()],
            OsStr::new("NEW"),
        );
        assert_eq!(entry.cmd, "/bin/new --ready");
    }

    #[test]
    fn refresh_evicts_dead_entries() {
        let mut collector = Collector::new();
        collector
            .cmd_cache
            .insert(u32::MAX, CmdEntry::new(0, &[], OsStr::new("dead")));
        let snapshot = collector.refresh();
        assert!(!collector.cmd_cache.contains_key(&u32::MAX));
        assert_eq!(collector.cmd_cache.len(), snapshot.processes.len());
    }
}
