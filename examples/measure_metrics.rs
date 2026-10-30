//! Dependency-free measurements of live collection and synthetic view workloads.
#![allow(dead_code)]

#[path = "../src/metrics/mod.rs"]
mod metrics;
#[path = "../src/process/mod.rs"]
mod process;

use std::{collections::HashSet, hint::black_box, time::Instant};

use metrics::collector::Collector;
use process::{
    model::{ProcessRow, SortColumn, filter_indices, sort_processes},
    tree::ProcessTree,
};

fn measure(label: &str, samples: usize, mut work: impl FnMut()) {
    let mut times = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        work();
        times.push(start.elapsed().as_secs_f64() * 1_000.0);
    }
    times.sort_by(f64::total_cmp);
    println!(
        "{label}: p50={:.3}ms p95={:.3}ms samples={samples}",
        times[samples / 2],
        times[(samples * 95 / 100).min(samples - 1)]
    );
}

fn main() {
    let mut collector = Collector::new();
    collector.warmup();
    let count = collector.refresh().processes.len();
    measure(&format!("live collection ({count} processes)"), 50, || {
        black_box(collector.refresh());
    });

    for count in [100, 1_000, 10_000] {
        let rows: Vec<_> = (1..=count)
            .map(|pid| {
                let name = format!("worker-{pid}");
                let cmd = format!("/usr/bin/{name} --queue jobs --token {pid}");
                ProcessRow {
                    pid,
                    start_time: u64::from(pid),
                    cpu: (pid % 97) as f32,
                    mem_bytes: u64::from(pid) * 4096,
                    cmd_lower: cmd.clone(),
                    cmd,
                    name_lower: name.clone(),
                    name,
                    parent_pid: (pid > 1).then_some(pid / 2),
                }
            })
            .collect();
        let expanded: HashSet<_> = (1..=count).collect();
        // Clone outside the timed section: measure sorting, not fixture allocation.
        let mut sorted = rows.clone();
        measure(
            &format!("sort/filter/tree ({count} processes)"),
            100,
            || {
                sorted.reverse();
                sort_processes(&mut sorted, SortColumn::Cpu, true, true);
                black_box(filter_indices(&sorted, "worker-1"));
                let tree = ProcessTree::build(&sorted);
                black_box(tree.flatten_visible(&expanded));
            },
        );
    }
}
