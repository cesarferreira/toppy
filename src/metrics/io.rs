use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

/// System disk and network throughput, in bytes per second.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IoSnapshot {
    pub disk_read: u64,
    pub disk_write: u64,
    pub net_rx: u64,
    pub net_tx: u64,
    /// False until one refresh interval has been measured.
    pub ready: bool,
}

pub fn bytes_per_second(bytes: u64, elapsed: Duration) -> u64 {
    let micros = elapsed.as_micros();
    if micros == 0 {
        return 0;
    }
    let rate = u128::from(bytes).saturating_mul(1_000_000) / micros;
    u64::try_from(rate).unwrap_or(u64::MAX)
}

/// `lo` on Linux and `lo0` on macOS. `en0` and other interface names stay included.
pub fn is_loopback(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("lo") else {
        return false;
    };
    rest.is_empty() || rest.bytes().all(|b| b.is_ascii_digit())
}

/// BSD device id for a macOS mount source.
///
/// `/dev/disk3s1s1` and `/dev/disk3s6` are slices of `disk3`. APFS volumes on that
/// disk each report the whole disk's IOKit counters, so they must share one id.
/// Names that are not `disk` plus digits are returned unchanged.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn physical_disk_id(mnt_from: &str) -> &str {
    let name = mnt_from.rsplit('/').next().unwrap_or(mnt_from);
    let Some(rest) = name.strip_prefix("disk") else {
        return name;
    };
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return name;
    }
    &name[.."disk".len() + digits]
}

/// Keep one sample per device. A second mount of the same device must not multiply the bytes.
///
/// When samples disagree, the larger combined read and write wins. On macOS the samples are
/// the same physical disk observed a moment apart.
pub fn sum_disk_deltas<K: Eq + Hash>(
    entries: impl IntoIterator<Item = (K, u64, u64)>,
) -> (u64, u64) {
    let mut chosen: HashMap<K, (u64, u64)> = HashMap::new();
    for (key, read_bytes, written_bytes) in entries {
        let slot = chosen.entry(key).or_insert((read_bytes, written_bytes));
        if read_bytes.saturating_add(written_bytes) > slot.0.saturating_add(slot.1) {
            *slot = (read_bytes, written_bytes);
        }
    }
    chosen.values().fold((0, 0), |(read, written), (r, w)| {
        (read.saturating_add(*r), written.saturating_add(*w))
    })
}

/// Sum receive and transmit deltas, skipping loopback interfaces.
pub fn sum_network_deltas<'a>(
    entries: impl IntoIterator<Item = (&'a str, u64, u64)>,
) -> (u64, u64) {
    let mut received = 0u64;
    let mut transmitted = 0u64;
    for (name, rx, tx) in entries {
        if is_loopback(name) {
            continue;
        }
        received = received.saturating_add(rx);
        transmitted = transmitted.saturating_add(tx);
    }
    (received, transmitted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_per_second_scales_with_the_interval() {
        assert_eq!(bytes_per_second(1_000, Duration::from_millis(500)), 2_000);
        assert_eq!(bytes_per_second(1_500, Duration::from_millis(1_500)), 1_000);
        assert_eq!(bytes_per_second(50, Duration::ZERO), 0);
        assert_eq!(
            bytes_per_second(u64::MAX, Duration::from_millis(1)),
            u64::MAX
        );
    }

    #[test]
    fn loopback_names_are_lo_and_numbered_lo() {
        assert!(is_loopback("lo"));
        assert!(is_loopback("lo0"));
        assert!(!is_loopback("eth0"));
        assert!(!is_loopback("en0"));
        assert!(!is_loopback("local"));
        assert!(!is_loopback("wlan0"));
    }

    #[test]
    fn disk_deltas_count_each_device_once() {
        let sda = "sda1";
        let sdb = "sdb1";
        assert_eq!(
            sum_disk_deltas([(sda, 100, 10), (sda, 100, 10), (sdb, 5, 7)]),
            (105, 17)
        );
    }

    #[test]
    fn duplicate_device_keeps_the_larger_sample() {
        let disk3 = "disk3";
        assert_eq!(sum_disk_deltas([(disk3, 0, 0), (disk3, 80, 15)]), (80, 15));
    }

    #[test]
    fn macos_apfs_volumes_share_one_physical_disk() {
        assert_eq!(physical_disk_id("/dev/disk3s1s1"), "disk3");
        assert_eq!(physical_disk_id("/dev/disk3s6"), "disk3");
        assert_eq!(physical_disk_id("/dev/disk10s2"), "disk10");
        assert_eq!(physical_disk_id("/dev/disk4"), "disk4");
        assert_eq!(physical_disk_id("devfs"), "devfs");

        let system = physical_disk_id("/dev/disk3s1s1");
        let data = physical_disk_id("/dev/disk3s6");
        let external = physical_disk_id("/dev/disk6s1");
        assert_eq!(
            sum_disk_deltas([(system, 1_000, 50), (data, 1_000, 50), (external, 8, 9)]),
            (1_008, 59)
        );
    }

    #[test]
    fn network_deltas_skip_loopback_on_linux_and_macos() {
        assert_eq!(
            sum_network_deltas([("lo", 9, 9), ("lo0", 4, 4), ("eth0", 20, 3), ("en0", 7, 1),]),
            (27, 4)
        );
    }
}
