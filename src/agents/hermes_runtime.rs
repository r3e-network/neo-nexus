//! NousResearch Hermes gateway contract (upstream b0ab2e163a50d4e6c36507eba955a6067fde6abc).
//! Its Windows drain uses a profile-scoped planned-stop marker, not SIGBREAK.
//! Invoke the upstream stop command so its process fingerprint units, lock and
//! service detection remain owned by Hermes.

use std::{
    fs::File,
    io::Read,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use serde_json::Value;

use super::{lifecycle::identity_status, AgentKind, AgentProfile, AgentRecord};
use crate::supervisor::{process_is_live, RecordedProcess};

const MAX_STATUS_BYTES: u64 = 1024 * 1024;
const STATUS_TTL_SECONDS: u64 = 120;

/// The caller falls back to supervised termination only on `Ok(false)`.
/// Identity errors are propagated, so a recycled PID is never signalled.
pub(super) fn request_stop(record: &AgentRecord, timeout: Duration) -> Result<bool> {
    if record.profile.kind != AgentKind::Hermes {
        return Ok(false);
    }
    let Some(pid) = record.pid else {
        return Ok(true);
    };
    match identity_status(record) {
        RecordedProcess::Gone => return Ok(true),
        RecordedProcess::Reused => bail!("Hermes process identity changed; graceful stop refused"),
        RecordedProcess::Alive => {}
    }
    let mut helper = stop_command(&record.profile)?
        .spawn()
        .context("cannot launch Hermes stop helper")?;
    wait_for_stop(&mut helper, pid, timeout)
}

fn stop_command(profile: &AgentProfile) -> Result<Command> {
    let home = profile
        .config_path
        .as_deref()
        .and_then(|path| path.parent())
        .context("Hermes stop requires a configured profile home")?;
    if !home.is_absolute() || !profile.binary_path.is_absolute() {
        bail!("Hermes stop paths must be absolute");
    }
    let mut command = Command::new(&profile.binary_path);
    command
        .args(["-m", "hermes_cli.main", "gateway", "stop"])
        .current_dir(&profile.working_dir)
        .envs(super::hermes::environment(profile))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    Ok(command)
}

fn wait_for_stop(helper: &mut Child, pid: u32, timeout: Duration) -> Result<bool> {
    let deadline = Instant::now() + timeout;
    loop {
        let status = match helper.try_wait() {
            Ok(status) => status,
            Err(error) => {
                let _ = helper.kill();
                let _ = helper.wait();
                return Err(error).context("cannot inspect Hermes stop helper");
            }
        };
        let gone = !process_is_live(pid);
        if gone || status.is_some_and(|status| !status.success()) || Instant::now() >= deadline {
            if status.is_none() {
                let _ = helper.kill();
                let _ = helper.wait();
            }
            // A zero helper exit is not evidence that the recorded target quit.
            return Ok(gone || !process_is_live(pid));
        }
        thread::sleep(Duration::from_millis(50));
    }
}

/// A bounded profile-scoped health observation. `None` means no attributable,
/// fresh snapshot, rather than an assertion that an idle process is unhealthy.
/// OS liveness/start-time validation remains the monitor's responsibility.
pub(super) fn runtime_health(
    profile: &AgentProfile,
    pid: u32,
    now_unix: u64,
) -> Result<Option<bool>> {
    let home = profile
        .config_path
        .as_deref()
        .and_then(|path| path.parent())
        .context("Hermes health requires a configured profile home")?;
    let file = match File::open(home.join("gateway_state.json")) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => bail!("cannot read Hermes runtime status"),
    };
    let mut bytes = Vec::new();
    file.take(MAX_STATUS_BYTES + 1)
        .read_to_end(&mut bytes)
        .context("cannot read Hermes runtime status")?;
    if bytes.len() as u64 > MAX_STATUS_BYTES {
        bail!("Hermes runtime status is too large");
    }
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| anyhow::anyhow!("Hermes runtime status is invalid JSON"))?;
    if value.get("pid").and_then(Value::as_u64) != Some(u64::from(pid)) || pid == 0 {
        return Ok(None);
    }
    if let Some(recorded_home) = value.get("hermes_home") {
        let Some(recorded_home) = recorded_home.as_str() else {
            return Ok(None);
        };
        let actual = std::path::Path::new(recorded_home).canonicalize().ok();
        if actual.is_none() || actual != home.canonicalize().ok() {
            return Ok(None);
        }
    }
    let Some(updated) = value
        .get("updated_at")
        .and_then(Value::as_str)
        .and_then(parse_utc_timestamp)
    else {
        return Ok(None);
    };
    if updated > now_unix.saturating_add(30)
        || now_unix.saturating_sub(updated) > STATUS_TTL_SECONDS
    {
        return Ok(None);
    }
    match value.get("gateway_state").and_then(Value::as_str) {
        Some("running") => {}
        Some("stopped" | "startup_failed" | "error") => return Ok(Some(false)),
        _ => return Ok(None),
    }
    if value
        .get("session_store")
        .and_then(|store| store.get("status"))
        .and_then(Value::as_str)
        .is_some_and(|status| matches!(status, "unavailable" | "retrying"))
    {
        return Ok(Some(false));
    }
    if let Some(platforms) = value.get("platforms") {
        let Some(platforms) = platforms.as_object() else {
            return Ok(None);
        };
        for platform in platforms.values() {
            // Ignore preserved state from an older process where provenance is
            // present. Only this writer's adapters may affect current health.
            if platform
                .get("writer_pid")
                .is_some_and(|writer| writer.as_u64() != Some(u64::from(pid)))
            {
                continue;
            }
            if platform
                .get("writer_start_time")
                .is_some_and(|start| Some(start) != value.get("start_time"))
            {
                continue;
            }
            if platform.get("needs_attention").and_then(Value::as_bool) == Some(true)
                || platform
                    .get("state")
                    .and_then(Value::as_str)
                    .is_some_and(|state| {
                        matches!(
                            state,
                            "error" | "failed" | "disconnected" | "retrying" | "reconnecting"
                        )
                    })
            {
                return Ok(Some(false));
            }
        }
    }
    Ok(Some(true))
}

// Hermes writes UTC datetime.isoformat(): support its +00:00 suffix and Z,
// optional fractional seconds, and validate every date/time component.
fn parse_utc_timestamp(text: &str) -> Option<u64> {
    let text = text
        .strip_suffix("+00:00")
        .or_else(|| text.strip_suffix('Z'))?;
    let (date, time) = text.split_once('T')?;
    let date: Vec<u32> = date
        .split('-')
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()
        .ok()?;
    if date.len() != 3 {
        return None;
    }
    let (year, month, day) = (date[0], date[1], date[2]);
    if !(1970..=9999).contains(&year) || !(1..=12).contains(&month) {
        return None;
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let month_days = [
        31,
        28 + u32::from(leap),
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if day == 0 || day > month_days[month as usize - 1] {
        return None;
    }
    let time = if let Some((whole, fraction)) = time.split_once('.') {
        if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        whole
    } else {
        time
    };
    let time: Vec<u32> = time
        .split(':')
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()
        .ok()?;
    if time.len() != 3 || time[0] > 23 || time[1] > 59 || time[2] > 59 {
        return None;
    }
    let leap_days_before = |year: u32| (year - 1) / 4 - (year - 1) / 100 + (year - 1) / 400;
    let days = (year - 1970) * 365 + leap_days_before(year) - leap_days_before(1970)
        + month_days[..month as usize - 1].iter().sum::<u32>()
        + day
        - 1;
    Some(u64::from(days) * 86400 + u64::from(time[0] * 3600 + time[1] * 60 + time[2]))
}

#[cfg(test)]
#[path = "../../tests/unit/agents/hermes_runtime.rs"]
mod tests;
