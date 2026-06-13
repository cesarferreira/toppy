const KIB: f64 = 1024.0;
const MIB: f64 = KIB * 1024.0;
const GIB: f64 = MIB * 1024.0;

pub fn format_bytes(bytes: u64) -> String {
    let b = bytes as f64;
    if b < MIB {
        format!("{:.1} KB", b / KIB)
    } else if b < GIB {
        format!("{:.1} MB", b / MIB)
    } else {
        format!("{:.2} GB", b / GIB)
    }
}

pub fn format_bytes_pair(used: u64, total: u64) -> String {
    format!("{} / {}", format_bytes(used), format_bytes(total))
}

pub fn percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (used as f32 / total as f32) * 100.0
    }
}

pub fn format_percent(pct: f32) -> String {
    format!("{pct:.1}%")
}
