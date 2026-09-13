use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};

use super::child::ManagedChild;
use crate::{
    child_environment::scrub_control_plane_environment,
    supervisor::{
        logging::{append_pid, open_launch_log},
        ManagedProcessSpec, ProcessStart,
    },
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
    let mut command = Command::new(&spec.binary_path);
    command
        .args(&spec.args)
        .current_dir(&spec.working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_log))
        .stderr(Stdio::from(stderr_log));
    scrub_control_plane_environment(&mut command);
    let child = command
        .spawn()
        .with_context(|| format!("failed to start {}", spec.binary_path.display()))?;
    let pid = child.id();
    append_pid(&mut log_file, &log_path, pid)?;
    // Everything past this point in the file is the child's own stdout/stderr.
    // A failure report quotes from here rather than guessing which lines were
    // the header. A file we just wrote to and cannot measure is not worth
    // failing a launch over; 0 degrades to "quote the whole file".
    let output_offset = log_file.metadata().map(|meta| meta.len()).unwrap_or(0);

    Ok((
        ManagedChild::new(
            child,
            log_path.clone(),
            spec.id.clone(),
            output_offset,
            spawned_at_unix(),
        ),
        ProcessStart { pid, log_path },
    ))
}

/// When a child was spawned, in unix seconds.
///
/// A clock that cannot be read gives 0, which the health layer reads as an
/// implausibly long uptime and therefore as "no startup grace" — the safe
/// direction, since it reports a silent node rather than excusing it.
fn spawned_at_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}
