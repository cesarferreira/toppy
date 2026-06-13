#[derive(Debug, Clone)]
pub struct ProcessRow {
    pub pid: u32,
    pub cpu: f32,
    pub mem_bytes: u64,
    pub cmd: String,
    pub parent_pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Pid,
    Cpu,
    Mem,
    Command,
}

impl SortColumn {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pid => "PID",
            Self::Cpu => "CPU",
            Self::Mem => "MEM",
            Self::Command => "CMD",
        }
    }
}

pub fn sort_processes(processes: &mut [ProcessRow], column: SortColumn, desc: bool) {
    processes.sort_by(|a, b| {
        let ord = match column {
            SortColumn::Pid => a.pid.cmp(&b.pid),
            SortColumn::Cpu => a
                .cpu
                .partial_cmp(&b.cpu)
                .unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Mem => a.mem_bytes.cmp(&b.mem_bytes),
            SortColumn::Command => a.cmd.to_lowercase().cmp(&b.cmd.to_lowercase()),
        };
        if desc { ord.reverse() } else { ord }
    });
}

pub fn filter_indices(processes: &[ProcessRow], filter: &str) -> Vec<usize> {
    if filter.is_empty() {
        return (0..processes.len()).collect();
    }
    let needle = filter.to_lowercase();
    processes
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            p.cmd.to_lowercase().contains(&needle) || p.pid.to_string().contains(&needle)
        })
        .map(|(i, _)| i)
        .collect()
}
