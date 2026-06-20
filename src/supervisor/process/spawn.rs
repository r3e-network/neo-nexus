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
) -> Result<(ManagedChild, ProcessStart)> {
    let mut log_file = open_launch_log(spec, &log_path)?;
    let stdout_log = log_file
        .try_clone()
        .with_context(|| format!("failed to clone process log {}", log_path.display()))?;
    let stderr_log = log_file
        .try_clone()
        .with_context(|| format!("failed to clone process log {}", log_path.display()))?;
    let child = Command::new(&spec.binary_path)
        .args(&spec.args)
        .current_dir(&spec.working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_log))
        .stderr(Stdio::from(stderr_log))
        .spawn()
        .with_context(|| format!("failed to start {}", spec.binary_path.display()))?;
    let pid = child.id();
    append_pid(&mut log_file, &log_path, pid)?;

    Ok((
        ManagedChild::new(child, log_path.clone()),
        ProcessStart { pid, log_path },
    ))
}
