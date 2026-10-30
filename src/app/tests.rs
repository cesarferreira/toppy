use super::*;

pub(crate) fn row(pid: u32, cpu: f32, parent_pid: Option<u32>) -> ProcessRow {
    ProcessRow {
        pid,
        start_time: u64::from(pid),
        cpu,
        parent_pid,
        mem_bytes: 4096,
        cmd: format!("worker-{pid} --jobs"),
        cmd_lower: format!("worker-{pid} --jobs"),
        name: format!("worker-{pid}"),
        name_lower: format!("worker-{pid}"),
    }
}

pub(crate) fn fixture() -> App {
    let mut app = App {
        processes: vec![
            row(10, 10.0, None),
            row(20, 90.0, Some(10)),
            row(30, 50.0, None),
        ],
        filtered_indices: Vec::new(),
        tree: ProcessTree::build(&[]),
        tree_expanded: HashSet::new(),
        flat_tree: Vec::new(),
        cpu: CpuSnapshot {
            per_core: vec![25.0],
            global: 25.0,
        },
        memory: MemorySnapshot {
            total: 8192,
            used: 4096,
            available: 4096,
            swap_total: 0,
            swap_used: 0,
        },
        warmed_up: true,
        sort: SortColumn::Cpu,
        sort_desc: true,
        selected: 0,
        scroll_offset: 0,
        filter: String::new(),
        mode: ViewMode::Table,
        overlay: None,
        refresh_ms: 1500,
        visible_rows: 10,
        cpu_cores_expanded: false,
        show_full_command: true,
        status_message: None,
        status_message_until: None,
        should_quit: false,
        collector: Collector::new(),
    };
    app.rebuild_view();
    app
}

#[test]
fn sort_keeps_selected_process() {
    let mut app = fixture();
    assert_eq!(app.selected_process().unwrap().pid, 20);
    app.set_sort(SortColumn::Pid);
    assert_eq!(app.selected_process().unwrap().pid, 20);
}

#[test]
fn refresh_keeps_selected_process_after_cpu_reordering() {
    let mut app = fixture();
    app.apply_snapshot(MetricsSnapshot {
        cpu: app.cpu.clone(),
        memory: app.memory.clone(),
        warmed_up: true,
        processes: vec![
            row(10, 99.0, None),
            row(20, 1.0, Some(10)),
            row(30, 50.0, None),
        ],
    });
    assert_eq!(app.selected_process().unwrap().pid, 20);
}

#[test]
fn tree_kill_menu_keeps_selected_process() {
    let mut app = fixture();
    app.toggle_tree();
    assert_eq!(app.selected_process().unwrap().pid, 10);
    app.open_kill_menu();
    assert_eq!(app.selected_process().unwrap().pid, 10);
    assert_eq!(
        app.overlay,
        Some(Overlay::KillMenu(ProcessIdentity {
            pid: 10,
            start_time: 10
        }))
    );
}

#[test]
fn help_returns_to_expanded_tree() {
    let mut app = fixture();
    app.toggle_tree();
    app.toggle_tree_node();
    app.move_selection(1);
    app.toggle_help();
    app.rebuild_view();
    app.exit_overlay();
    assert_eq!(app.mode, ViewMode::Tree);
    assert_eq!(app.selected_process().unwrap().pid, 20);
    assert!(app.tree_expanded.contains(&10));
}

#[test]
fn filter_returns_to_tree() {
    let mut app = fixture();
    app.toggle_tree();
    app.enter_filter();
    app.push_filter_char('3');
    app.exit_overlay();
    assert_eq!(app.mode, ViewMode::Tree);
}

#[test]
fn filter_in_tree_keeps_matching_child_and_its_ancestors() {
    let mut app = fixture();
    app.toggle_tree();
    app.enter_filter();
    for ch in "worker-20".chars() {
        app.push_filter_char(ch);
    }
    app.exit_overlay();
    assert_eq!(
        app.flat_tree.iter().map(|row| row.0).collect::<Vec<_>>(),
        vec![10, 20]
    );
    app.clear_filter();
    assert_eq!(
        app.flat_tree.iter().map(|row| row.0).collect::<Vec<_>>(),
        vec![10, 30]
    );
}

#[test]
fn kill_target_stays_pinned_when_list_changes() {
    let mut app = fixture();
    app.open_kill_menu();
    app.apply_snapshot(MetricsSnapshot {
        cpu: app.cpu.clone(),
        memory: app.memory.clone(),
        warmed_up: true,
        processes: vec![row(10, 99.0, None)],
    });
    assert_eq!(
        app.overlay,
        Some(Overlay::KillMenu(ProcessIdentity {
            pid: 20,
            start_time: 20
        }))
    );
}

#[test]
fn empty_snapshot_resets_selection_without_losing_kill_target() {
    let mut app = fixture();
    app.move_end();
    app.open_kill_menu();
    app.apply_snapshot(MetricsSnapshot {
        cpu: app.cpu.clone(),
        memory: app.memory.clone(),
        warmed_up: true,
        processes: vec![],
    });
    assert!(app.selected_process().is_none());
    assert_eq!(app.selected, 0);
    assert_eq!(app.scroll_offset, 0);
    assert!(matches!(app.overlay, Some(Overlay::KillMenu(_))));
}

#[test]
fn action_feedback_survives_an_immediate_scheduled_tick() {
    let mut app = fixture();
    app.slower_refresh();
    app.tick();
    assert_eq!(app.status_message.as_deref(), Some("refresh 2000ms"));
}

#[test]
fn feedback_expires_without_waiting_for_another_tick() {
    let mut app = fixture();
    app.slower_refresh();
    assert_eq!(app.status_message(), Some("refresh 2000ms"));
    app.status_message_until = Some(Instant::now() - Duration::from_millis(1));
    assert_eq!(app.status_message(), None);
}

#[test]
fn refresh_preserves_expansion_only_for_same_live_identities() {
    let mut app = fixture();
    app.tree_expanded = [10, 20, 30].into_iter().collect();
    let mut reused = row(10, 10.0, None);
    reused.start_time = 11;
    app.apply_snapshot(MetricsSnapshot {
        cpu: app.cpu.clone(),
        memory: app.memory.clone(),
        warmed_up: true,
        processes: vec![reused, row(20, 90.0, Some(10))],
    });
    assert_eq!(app.tree_expanded, [20].into_iter().collect());
}

#[test]
#[ignore = "manual performance measurement; run in release mode with --nocapture"]
fn measure_expanded_snapshot_refresh() {
    use std::time::Instant;
    let mut app = fixture();
    let processes: Vec<_> = (1..=10_000)
        .map(|pid| row(pid, (pid % 97) as f32, (pid > 1).then_some(pid / 2)))
        .collect();
    app.apply_snapshot(MetricsSnapshot {
        cpu: app.cpu.clone(),
        memory: app.memory.clone(),
        warmed_up: true,
        processes: processes.clone(),
    });
    app.toggle_tree();
    app.tree_expanded = (1..=10_000).collect();
    app.rebuild_view();
    let mut times = Vec::new();
    for _ in 0..20 {
        let snapshot = MetricsSnapshot {
            cpu: app.cpu.clone(),
            memory: app.memory.clone(),
            warmed_up: true,
            processes: processes.clone(),
        };
        let start = Instant::now();
        app.apply_snapshot(snapshot);
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(f64::total_cmp);
    println!(
        "full app refresh (10k processes, all expanded): p50={:.3}ms p95={:.3}ms samples=20",
        times[10], times[19]
    );
    assert_eq!(app.flat_tree.len(), 10_000);
}
