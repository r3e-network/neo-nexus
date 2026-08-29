use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    process::{Child, ExitStatus},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};

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

/// Liveness checks refresh only the watched pid, so this can track the
/// handle-based poll closely without scanning the whole process table.
const UNMANAGED_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Stop a process this supervisor holds no handle for.
///
/// A node started by an earlier `--node-start`, or one still running after a
/// server restart, is recorded with a pid but has no `Child` anywhere in this
/// process. Marking such a row `Stopped` without killing anything would leave
/// the node running while the workbench said it was not.
///
/// `None` means the pid was already gone, so the caller can settle the recorded
/// status without claiming to have stopped something.
pub(super) fn stop_by_pid(
    process_id: &str,
    pid: u32,
    log_path: PathBuf,
    grace_period: Duration,
) -> Option<ProcessStop> {
    let mut system = sysinfo::System::new();
    // Already gone: there is nothing here to stop.
    live_process(&mut system, pid)?;

    let graceful_requested = request_graceful_termination(pid).is_ok();
    // As in `stop_child`: no signal sent means nothing to wait for.
    let deadline = Instant::now() + grace_period;
    while graceful_requested && live_process(&mut system, pid).is_some() {
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(UNMANAGED_POLL_INTERVAL);
    }

    let forced = live_process(&mut system, pid).is_some();
    if forced {
        live_process(&mut system, pid)?.kill();
        // No handle to wait on, so the exit code is genuinely unknowable rather
        // than absent by accident; `append_stop_log` records it as a signal.
        let kill_deadline = Instant::now() + grace_period;
        while live_process(&mut system, pid).is_some() && Instant::now() < kill_deadline {
            thread::sleep(UNMANAGED_POLL_INTERVAL);
        }
    }

    let stop = ProcessStop {
        process_id: process_id.to_string(),
        pid,
        log_path,
        graceful: graceful_requested && !forced,
        forced,
        exit_code: None,
    };
    append_stop_log(&stop, grace_period);
    Some(stop)
}

fn live_process(system: &mut sysinfo::System, pid: u32) -> Option<&sysinfo::Process> {
    let target = sysinfo::Pid::from_u32(pid);
    system.refresh_processes(
        sysinfo::ProcessesToUpdate::Some(&[target]),
        // Without this, an exited pid would linger as a stale entry and the
        // loop below would wait out its grace period for nothing.
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
