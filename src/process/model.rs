#[derive(Debug, Clone)]
pub struct ProcessRow {
    pub pid: u32,
    pub cpu: f32,
    pub mem_bytes: u64,
    pub cmd: String,
    pub cmd_lower: String,
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
            SortColumn::Command => a.cmd_lower.cmp(&b.cmd_lower),
        };
        if desc { ord.reverse() } else { ord }
    });
}

pub fn filter_indices(processes: &[ProcessRow], filter: &str) -> Vec<usize> {
    if filter.is_empty() {
        return (0..processes.len()).collect();
    }
    let mut needle = filter.to_string();
    needle.make_ascii_lowercase();
    let needle_is_digits = !needle.is_empty() && needle.bytes().all(|b| b.is_ascii_digit());
    let mut pid_buf = [0u8; 10];
    processes
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            if p.cmd_lower.contains(&needle) {
                return true;
            }
            if needle_is_digits {
                let s = format_pid(&mut pid_buf, p.pid);
                s.contains(&needle)
            } else {
                false
            }
        })
        .map(|(i, _)| i)
        .collect()
}

fn format_pid(buf: &mut [u8; 10], pid: u32) -> &str {
    let mut n = pid;
    let mut i = buf.len();
    if n == 0 {
        i -= 1;
        buf[i] = b'0';
    } else {
        while n > 0 {
            i -= 1;
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }
    std::str::from_utf8(&buf[i..]).unwrap_or("")
}
