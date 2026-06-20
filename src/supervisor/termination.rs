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
    let deadline = Instant::now() + grace_period;
    let (forced, status) = match wait_until_exit(child, deadline)? {
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
