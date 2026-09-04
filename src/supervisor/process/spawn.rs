use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};

use super::child::ManagedChild;
use crate::supervisor::{
    logging::{append_pid, open_launch_log},
    ManagedProcessSpec, ProcessStart,
};

pub(super) fn spawn_managed_child(
    spec: &ManagedProcessSpec,
    log_path: PathBuf,
    environment: &[(String, String)],
) -> Result<(ManagedChild, ProcessStart)> {
    let mut log_file = open_launch_log(spec, &log_path)?;
    let stdout_log = log_file
        .try_clone()
        .with_context(|| format!("failed to clone process log {}", log_path.display()))?;
    let stderr_log = log_file
        .try_clone()
        .with_context(|| format!("failed to clone process log {}", log_path.display()))?;
    let mut command = Command::new(&spec.binary_path);
    command
        .args(&spec.args)
        .envs(environment.iter().map(|(name, value)| (name, value)))
        .current_dir(&spec.working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_log))
        .stderr(Stdio::from(stderr_log));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        crate::supervisor::windows_console::prevent_standard_pipe_inheritance()?;
        // The child PID is its console process-group ID. CTRL_BREAK can then
        // target this node without broadcasting to the workbench or other nodes.
        command.creation_flags(0x0000_0200); // CREATE_NEW_PROCESS_GROUP
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("failed to start {}", spec.binary_path.display()))?;
    let pid = child.id();
    if let Err(error) = append_pid(&mut log_file, &log_path, pid) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }

    Ok((
        ManagedChild::new(child, log_path.clone(), spec.id.clone()),
        ProcessStart { pid, log_path },
    ))
}
