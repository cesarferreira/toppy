use anyhow::{Context, Result};
use sysinfo::{Pid, Signal, System};

pub fn kill_process(system: &System, pid: u32, signal: Signal) -> Result<()> {
    let pid = Pid::from_u32(pid);
    let process = system
        .process(pid)
        .context("process not found")?;

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
