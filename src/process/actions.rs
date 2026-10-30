use anyhow::{Context, Result};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System};

use super::model::ProcessIdentity;

pub fn kill_process(target: ProcessIdentity, signal: Signal) -> Result<()> {
    // Re-read this PID immediately before signaling rather than trusting a UI snapshot.
    let pid = Pid::from_u32(target.pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().without_tasks(),
    );
    let process = system.process(pid).context("process not found")?;

    anyhow::ensure!(
        process.start_time() == target.start_time,
        "process identity changed (PID reused); no signal sent"
    );

    match process.kill_with(signal) {
        Some(true) => Ok(()),
        Some(false) => anyhow::bail!("failed to send signal"),
        None => anyhow::bail!("signal not supported on this platform"),
    }
}

pub fn signal_label(signal: Signal) -> &'static str {
    match signal {
        Signal::Term => "TERM",
        Signal::Kill => "KILL",
        _ => "SIGNAL",
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{
        process::{Child, Command},
        thread,
        time::{Duration, Instant},
    };

    struct ChildGuard(Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn child() -> (ChildGuard, ProcessIdentity) {
        let child = ChildGuard(Command::new("sleep").arg("30").spawn().unwrap());
        let pid = Pid::from_u32(child.0.id());
        let mut system = System::new();
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().without_tasks(),
        );
        let target = ProcessIdentity {
            pid: pid.as_u32(),
            start_time: system.process(pid).unwrap().start_time(),
        };
        (child, target)
    }

    #[test]
    fn stale_identity_does_not_signal_live_process() {
        let (mut child, mut target) = child();
        target.start_time = target.start_time.saturating_sub(1);
        let error = kill_process(target, Signal::Term).unwrap_err();
        assert!(error.to_string().contains("identity changed"));
        assert!(child.0.try_wait().unwrap().is_none());
    }

    #[test]
    fn matching_identity_can_terminate_owned_child() {
        let (mut child, target) = child();
        kill_process(target, Signal::Term).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                use std::os::unix::process::ExitStatusExt;
                assert_eq!(status.signal(), Some(15));
                break;
            }
            assert!(Instant::now() < deadline, "child did not receive TERM");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn exited_process_is_reported_instead_of_retargeted() {
        let (mut child, target) = child();
        child.0.kill().unwrap();
        child.0.wait().unwrap();
        assert!(
            kill_process(target, Signal::Kill)
                .unwrap_err()
                .to_string()
                .contains("process not found")
        );
    }
}
