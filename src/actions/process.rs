use anyhow::{Result, bail};
use sysinfo::{Pid, ProcessesToUpdate, Signal, System};

pub fn kill_process(pid: u32) -> Result<()> {
    if pid == std::process::id() {
        bail!("refusing to kill argus itself pid: {pid}");
    }

    let sys_pid = Pid::from_u32(pid);
    let mut system = System::new();

    // select only one specific process
    system.refresh_processes(ProcessesToUpdate::Some(&[sys_pid]), true);

    let Some(process) = system.process(sys_pid) else {
        bail!("process pid: {sys_pid} not found");
    };

    let term_sent = process.kill_with(Signal::Term).unwrap_or(false);
    if term_sent {
        return Ok(());
    }

    let kill_sent = process.kill_with(Signal::Kill).unwrap_or(false);

    if kill_sent {
        return Ok(());
    }

    bail!("failed to kill process: {pid}, probably permission denied");
}
