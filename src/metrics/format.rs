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

/// Compact label for meter suffixes, e.g. `18.9G` like htop.
pub fn format_bytes_compact(bytes: u64) -> String {
    let b = bytes as f64;
    if b < MIB {
        format!("{:.0}K", b / KIB)
    } else if b < GIB {
        format!("{:.1}M", b / MIB)
    } else {
        format!("{:.1}G", b / GIB)
    }
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

/// Throughput label, e.g. `12.0 MB/s`. Idle traffic stays `0 B/s`.
pub fn format_rate(bytes_per_sec: u64) -> String {
    if bytes_per_sec == 0 {
        return "0 B/s".to_string();
    }
    format!("{}/s", format_bytes(bytes_per_sec))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_rate_uses_byte_units_and_a_per_second_suffix() {
        assert_eq!(format_rate(0), "0 B/s");
        assert_eq!(format_rate(1024), "1.0 KB/s");
        assert_eq!(format_rate(12 * 1024 * 1024), "12.0 MB/s");
    }
}
