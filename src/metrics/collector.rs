use std::collections::HashMap;
use std::ffi::OsStr;

use sysinfo::{ProcessesToUpdate, System};

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
    cmd: String,
    cmd_lower: String,
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
        system.refresh_processes(ProcessesToUpdate::All, true);
        Self {
            system,
            tick_count: 0,
            cmd_cache: HashMap::new(),
        }
    }

    pub fn warmup(&mut self) {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.system.refresh_cpu_usage();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.tick_count = 1;
    }

    pub fn refresh(&mut self) -> MetricsSnapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
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
        let mut alive: Vec<u32> = Vec::with_capacity(sys_procs.len());

        for (pid, process) in sys_procs {
            let pid_u32 = pid.as_u32();
            alive.push(pid_u32);

            let cmd_entry = self.cmd_cache.entry(pid_u32).or_insert_with(|| {
                let cmd = build_cmd_string(process.cmd(), process.name());
                let mut cmd_lower = cmd.clone();
                cmd_lower.make_ascii_lowercase();
                CmdEntry { cmd, cmd_lower }
            });

            processes.push(ProcessRow {
                pid: pid_u32,
                cpu: process.cpu_usage(),
                mem_bytes: process.memory(),
                cmd: cmd_entry.cmd.clone(),
                cmd_lower: cmd_entry.cmd_lower.clone(),
                parent_pid: process.parent().map(|p| p.as_u32()),
            });
        }

        // Evict dead pids from the cmd cache.
        if self.cmd_cache.len() > alive.len() {
            let alive_set: std::collections::HashSet<u32> = alive.into_iter().collect();
            self.cmd_cache.retain(|pid, _| alive_set.contains(pid));
        }

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

    pub fn system(&self) -> &System {
        &self.system
    }
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
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
