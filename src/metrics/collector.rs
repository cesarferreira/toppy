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

pub struct Collector {
    system: System,
    tick_count: u32,
}

impl Collector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self {
            system,
            tick_count: 0,
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

        let per_core: Vec<f32> = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .collect();

        let global = if per_core.is_empty() {
            0.0
        } else {
            per_core.iter().sum::<f32>() / per_core.len() as f32
        };

        let processes: Vec<ProcessRow> = self
            .system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let cmd = {
                    let parts: Vec<String> = process
                        .cmd()
                        .iter()
                        .map(|s| s.to_string_lossy().into_owned())
                        .collect();
                    if parts.is_empty() {
                        process.name().to_string_lossy().into_owned()
                    } else {
                        parts.join(" ")
                    }
                };

                ProcessRow {
                    pid: pid.as_u32(),
                    cpu: process.cpu_usage(),
                    mem_bytes: process.memory(),
                    cmd,
                    parent_pid: process.parent().map(|p| p.as_u32()),
                }
            })
            .collect();

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
