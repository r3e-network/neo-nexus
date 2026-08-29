use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, ExitStatus},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};

use crate::types::NodeConfig;

use super::{model::unix_timestamp, ProcessStop};

const STOP_POLL_INTERVAL: Duration = Duration::from_millis(25);

pub(super) fn stop_child(
    process_id: &str,
    child: &mut Child,
    log_path: PathBuf,
    grace_period: Duration,
) -> Result<ProcessStop> {
    let pid = child.id();
    let graceful_requested = request_graceful_termination(pid).is_ok();
    // Waiting for an exit nobody requested just makes the operator watch a
    // timeout, and guarantees the "forced" label. Graceful shutdown is only
    // worth waiting for where a signal was actually delivered.
    let deadline = Instant::now() + grace_period;
    let exited = graceful_requested
        .then(|| wait_until_exit(child, deadline))
        .transpose()?
        .flatten();
    let (forced, status) = match exited {
        Some(status) => (false, status),
        None => {
            child
                .kill()
                .with_context(|| format!("failed to force stop process {process_id}"))?;
            (
                true,
                child
                    .wait()
                    .with_context(|| format!("failed to wait for forced stop of {process_id}"))?,
            )
        }
    };

    let stop = ProcessStop {
        process_id: process_id.to_string(),
        pid,
        log_path,
        graceful: graceful_requested && !forced,
        forced,
        exit_code: status.code(),
    };
    append_stop_log(&stop, grace_period);
    Ok(stop)
}

fn wait_until_exit(child: &mut Child, deadline: Instant) -> Result<Option<ExitStatus>> {
    loop {
        if let Some(status) = child
            .try_wait()
            .context("failed to inspect stopping process")?
        {
            return Ok(Some(status));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        thread::sleep(STOP_POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())));
    }
}

/// Coarser than the handle-based poll: each check consults the process table
/// rather than a handle we already own.
const UNMANAGED_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// What reaching a pid without a handle actually accomplished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PidStop {
    /// The recorded process was signalled and is gone.
    Stopped(ProcessStop),
    /// Nothing runs under that pid; the recorded status is simply stale.
    AlreadyGone,
    /// Some other process owns the pid now. Nothing was signalled.
    PidReused,
}

/// Stop a process this supervisor holds no handle for.
///
/// A node started by an earlier `--node-start`, or one still running after a
/// server restart, is recorded with a pid but has no `Child` anywhere in this
/// process. Marking such a row `Stopped` without killing anything would leave
/// the node running while the workbench said it was not.
///
/// The pid is only trusted after the process behind it is checked against the
/// binary the workspace recorded. Pids are reused, and signalling whatever
/// happens to hold the number now is far worse than refusing to act.
pub(super) fn stop_by_pid(node: &NodeConfig, log_path: PathBuf, grace_period: Duration) -> PidStop {
    let Some(pid) = node.pid else {
        return PidStop::AlreadyGone;
    };
    let mut system = sysinfo::System::new();
    match identify_recorded_process(&mut system, node) {
        Some(_) => {}
        None if !process_is_live(pid) => return PidStop::AlreadyGone,
        // Something answers to that pid, but it is not our node.
        None => return PidStop::PidReused,
    }

    let graceful_requested = request_graceful_termination(pid).is_ok();
    // As in `stop_child`: no signal sent means nothing to wait for.
    let deadline = Instant::now() + grace_period;
    while graceful_requested && process_is_live(pid) {
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(UNMANAGED_POLL_INTERVAL);
    }

    let forced = process_is_live(pid);
    if forced {
        identify_recorded_process(&mut system, node).map(sysinfo::Process::kill);
        // No handle to wait on, so the exit code is genuinely unknowable rather
        // than absent by accident; `append_stop_log` records it as a signal.
        let kill_deadline = Instant::now() + grace_period;
        while process_is_live(pid) && Instant::now() < kill_deadline {
            thread::sleep(UNMANAGED_POLL_INTERVAL);
        }
    }

    let stop = ProcessStop {
        process_id: node.id.clone(),
        pid,
        log_path,
        graceful: graceful_requested && !forced,
        forced,
        exit_code: None,
    };
    append_stop_log(&stop, grace_period);
    PidStop::Stopped(stop)
}

/// The live process behind the node's recorded pid, but only if it is the
/// binary the workspace says it should be.
fn identify_recorded_process<'a>(
    system: &'a mut sysinfo::System,
    node: &NodeConfig,
) -> Option<&'a sysinfo::Process> {
    let pid = node.pid?;
    let process = live_process(system, pid)?;
    process_matches_binary(process, &node.binary_path).then_some(process)
}

fn process_matches_binary(process: &sysinfo::Process, binary_path: &Path) -> bool {
    name_matches_binary(&process.name().to_string_lossy(), binary_path)
}

/// Whether a name the OS reports is the executable at `binary_path`.
///
/// Comparison is by file stem, case-insensitively, tolerating the `.exe` the
/// Windows process list appends and the extension a recorded path may lack.
/// A path with no usable stem is refused: guessing at identity is exactly what
/// this check exists to avoid.
pub fn name_matches_binary(reported: &str, binary_path: &Path) -> bool {
    let Some(stem) = binary_path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };
    let reported = reported.trim().to_ascii_lowercase();
    let expected = stem.to_ascii_lowercase();
    reported == expected || reported == format!("{expected}.exe")
}

/// Whether any process currently answers to this pid.
///
/// `kill(pid, 0)` is the portable-by-convention probe on Unix: it performs the
/// permission and existence checks without delivering anything. `EPERM` still
/// means the process exists, just not for us to signal.
#[cfg(unix)]
pub fn process_is_live(pid: u32) -> bool {
    let probe = unsafe { libc::kill(pid as libc::pid_t, 0) };
    if probe == 0 {
        return true;
    }
    std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// Windows has no `kill(2)`; ask the process list instead.
#[cfg(not(unix))]
pub fn process_is_live(pid: u32) -> bool {
    let mut system = sysinfo::System::new();
    live_process(&mut system, pid).is_some()
}

/// Which of `pids` are alive right now.
///
/// On Windows each probe rebuilds the process table, so a caller that asks
/// about several pids must be served by one scan rather than many.
#[cfg(not(unix))]
pub fn live_pids(pids: &[u32]) -> std::collections::BTreeSet<u32> {
    let mut alive = std::collections::BTreeSet::new();
    if pids.is_empty() {
        return alive;
    }
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    for pid in pids {
        if system.process(sysinfo::Pid::from_u32(*pid)).is_some() {
            alive.insert(*pid);
        }
    }
    alive
}

/// `kill(pid, 0)` is cheap enough to ask one at a time.
#[cfg(unix)]
pub fn live_pids(pids: &[u32]) -> std::collections::BTreeSet<u32> {
    pids.iter()
        .copied()
        .filter(|&pid| process_is_live(pid))
        .collect()
}

fn live_process(system: &mut sysinfo::System, pid: u32) -> Option<&sysinfo::Process> {
    let target = sysinfo::Pid::from_u32(pid);
    system.refresh_processes(
        // A scoped refresh looks only at processes this (fresh, empty) `System`
        // already knows, which finds nothing on macOS; scan the table.
        sysinfo::ProcessesToUpdate::All,
        // Without this, an exited pid would linger as a stale entry and the
        // loop above would wait out its grace period for nothing.
        true,
    );
    system.process(target)
}

fn append_stop_log(stop: &ProcessStop, grace_period: Duration) {
    let Ok(mut log_file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stop.log_path)
    else {
        return;
    };
    let mode = if stop.forced {
        "forced"
    } else if stop.graceful {
        "graceful"
    } else {
        "stopped"
    };
    let exit_code = stop
        .exit_code
        .map_or_else(|| "signal".to_string(), |code| code.to_string());
    let _ = writeln!(log_file, "\n== NeoNexus stop {} ==", unix_timestamp());
    let _ = writeln!(log_file, "process-id: {}", stop.process_id);
    let _ = writeln!(log_file, "pid: {}", stop.pid);
    let _ = writeln!(log_file, "stop-mode: {mode}");
    let _ = writeln!(log_file, "exit-code: {exit_code}");
    let _ = writeln!(log_file, "grace-period-ms: {}", grace_period.as_millis());
}

#[cfg(unix)]
fn request_graceful_termination(pid: u32) -> std::io::Result<()> {
    let result = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
    if result == 0 {
        Ok(())
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            Ok(())
        } else {
            Err(error)
        }
    }
}

#[cfg(not(unix))]
fn request_graceful_termination(_pid: u32) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "graceful process termination is not available on this platform",
    ))
}

#[cfg(test)]
#[path = "../../tests/unit/supervisor/termination/tests.rs"]
mod tests;
