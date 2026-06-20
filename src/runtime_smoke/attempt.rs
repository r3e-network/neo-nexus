use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};

use crate::{
    argv::format_command,
    redaction::{redact_sensitive_args, redact_sensitive_text},
};

use super::RuntimeSmokeAttempt;

const OUTPUT_LIMIT_BYTES: u64 = 16 * 1024;
static OUTPUT_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn run_attempt(
    command_path: &Path,
    args: &[String],
    timeout: Duration,
) -> Result<RuntimeSmokeAttempt> {
    let stdout_path = temp_output_path("stdout");
    let stderr_path = temp_output_path("stderr");
    let stdout_file = File::create(&stdout_path)
        .with_context(|| format!("failed to create {}", stdout_path.display()))?;
    let stderr_file = File::create(&stderr_path)
        .with_context(|| format!("failed to create {}", stderr_path.display()))?;

    let started = Instant::now();
    let mut child = Command::new(command_path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .with_context(|| format!("failed to run {}", command_path.display()))?;

    let mut timed_out = false;
    let exit_code = loop {
        if let Some(status) = child
            .try_wait()
            .with_context(|| format!("failed to inspect {}", command_path.display()))?
        {
            break status.code();
        }

        if started.elapsed() >= timeout {
            timed_out = true;
            let _ = child.kill();
            let status = child
                .wait()
                .with_context(|| format!("failed to collect {}", command_path.display()))?;
            break status.code();
        }

        thread::sleep(Duration::from_millis(20));
    };

    let elapsed_ms = started.elapsed().as_millis();
    let stdout = redact_sensitive_text(&read_limited_text(&stdout_path));
    let stderr = redact_sensitive_text(&read_limited_text(&stderr_path));
    let _ = fs::remove_file(&stdout_path);
    let _ = fs::remove_file(&stderr_path);

    Ok(RuntimeSmokeAttempt {
        command_line: format_command(command_path, &redact_sensitive_args(args)),
        exit_code,
        timed_out,
        elapsed_ms,
        stdout,
        stderr,
    })
}

fn read_limited_text(path: &Path) -> String {
    let Ok(file) = File::open(path) else {
        return String::new();
    };
    let mut buffer = Vec::new();
    let _ = file
        .take(OUTPUT_LIMIT_BYTES)
        .read_to_end(&mut buffer)
        .map(|_| ());
    String::from_utf8_lossy(&buffer).trim().to_string()
}

fn temp_output_path(kind: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let sequence = OUTPUT_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "neonexus-runtime-smoke-{}-{timestamp}-{sequence}-{kind}.log",
        std::process::id(),
    ))
}
