use super::{
    profile::{code_digest, digest, now, process_started_at},
    AgentProfile, AgentRecord, AgentStatus,
};
use crate::{
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    supervision::EngineState,
    supervisor::{recorded_process, PidStop, RecordedProcess},
};
use anyhow::{bail, Context, Result};

pub(super) fn load(state: &EngineState, id: &str) -> Result<AgentRecord> {
    state
        .repository
        .list_agents()?
        .into_iter()
        .find(|agent| agent.profile.id == id)
        .context("agent not found")
}

pub fn save(state: &EngineState, mut profile: AgentProfile) -> Result<()> {
    let supervisor = state.supervisor.lock().unwrap_or_else(|e| e.into_inner());
    if profile.id.is_empty() {
        profile.id = uuid::Uuid::new_v4().to_string();
    }
    if supervisor.is_managing(&profile.process_id()) {
        bail!("stop the agent before changing its profile");
    }
    if let Some(existing) = state
        .repository
        .list_agents()?
        .into_iter()
        .find(|record| record.profile.id == profile.id)
    {
        if existing.pid.is_some() || existing.desired_running {
            bail!("stop the agent before changing its profile");
        }
    }
    profile.validate(&state.repository)?;
    profile.binary_path = profile.binary_path.canonicalize()?;
    profile.working_dir = profile.working_dir.canonicalize()?;
    profile.config_path = profile
        .config_path
        .map(|path| path.canonicalize())
        .transpose()?;
    profile.binary_sha256 = code_digest(&profile)?;
    profile.config_sha256 = profile.config_path.as_deref().map(digest).transpose()?;
    let record = AgentRecord {
        profile,
        status: AgentStatus::Stopped,
        pid: None,
        process_started_at: None,
        desired_running: false,
        restart_attempts: 0,
        restart_after: None,
        healthy: None,
        last_health_at: 0,
    };
    state.repository.put_agent(&record)?;
    journal(
        state,
        &record,
        EventKind::AgentSaved,
        EventSeverity::Info,
        "profile saved; executable and configuration fingerprints updated",
    );
    Ok(())
}

pub fn start(state: &EngineState, id: &str) -> Result<()> {
    start_inner(state, id, true)
}

pub(super) fn start_inner(state: &EngineState, id: &str, manual: bool) -> Result<()> {
    let mut supervisor = state.supervisor.lock().unwrap_or_else(|e| e.into_inner());
    let mut record = load(state, id)?;
    if !manual && (!record.desired_running || record.restart_after.is_none_or(|at| at > now())) {
        return Ok(());
    }
    if record.pid.is_some() || supervisor.is_managing(&record.profile.process_id()) {
        bail!("agent already has a recorded process; stop it before starting again");
    }
    if manual {
        record.restart_attempts = 0;
    }
    record.desired_running = true;
    let outcome = (|| {
        record.profile.validate(&state.repository)?;
        if code_digest(&record.profile)? != record.profile.binary_sha256 {
            bail!("agent executable or source changed: review its version and save the stopped profile before starting");
        }
        if record
            .profile
            .config_path
            .as_deref()
            .map(digest)
            .transpose()?
            != record.profile.config_sha256
        {
            bail!(
                "agent configuration changed: review and save the stopped profile before starting"
            );
        }
        let spec = record.profile.spec(&state.repository)?;
        let environment = if record.profile.kind == super::AgentKind::Hermes {
            super::hermes::environment(&record.profile)
        } else {
            vec![]
        };
        let started =
            supervisor.start_process_with_env(&spec, log_path(state, id), &environment)?;
        record.pid = Some(started.pid);
        record.process_started_at = process_started_at(started.pid);
        record.status = AgentStatus::Running;
        record.restart_after = None;
        record.healthy = None;
        record.last_health_at = 0;
        if let Err(error) = state.repository.put_agent(&record) {
            let _ = supervisor.stop_process(&spec.id);
            return Err(error);
        }
        Ok(())
    })();
    if let Err(error) = outcome {
        record.pid = None;
        record.process_started_at = None;
        record.status = AgentStatus::Error;
        schedule(&mut record);
        state.repository.put_agent(&record)?;
        journal(
            state,
            &record,
            EventKind::AgentStartFailed,
            EventSeverity::Critical,
            &format!("start failed: {error}"),
        );
        return Err(error);
    }
    journal(
        state,
        &record,
        EventKind::AgentStarted,
        EventSeverity::Info,
        "process started",
    );
    Ok(())
}

pub fn stop(state: &EngineState, id: &str) -> Result<()> {
    let mut supervisor = state.supervisor.lock().unwrap_or_else(|e| e.into_inner());
    let mut record = load(state, id)?;
    if record.profile.kind == super::AgentKind::Hermes && record.pid.is_some() {
        super::hermes_runtime::request_stop(&record, std::time::Duration::from_secs(20))?;
    }
    let mut forced = false;
    match supervisor.stop_process(&record.profile.process_id())? {
        Some(stopped) => {
            forced = stopped.forced;
        }
        None => {
            if identity_status(&record) == RecordedProcess::Reused {
                bail!("recorded PID identity changed; no process was stopped");
            }
            match supervisor
                .stop_recorded_pid(&record.profile.identity(record.pid), log_path(state, id))?
            {
                PidStop::Stopped(stopped) => {
                    forced = stopped.forced;
                }
                PidStop::AlreadyGone => (),
                PidStop::PidReused => bail!(
                    "recorded PID does not match the agent executable; no process was stopped"
                ),
            }
        }
    }
    record.status = AgentStatus::Stopped;
    record.pid = None;
    record.process_started_at = None;
    record.desired_running = false;
    record.restart_after = None;
    record.restart_attempts = 0;
    record.healthy = None;
    state.repository.put_agent(&record)?;
    journal(
        state,
        &record,
        EventKind::AgentStopped,
        if forced {
            EventSeverity::Warning
        } else {
            EventSeverity::Info
        },
        if forced {
            "process force-stopped after grace period; automatic restart cancelled"
        } else {
            "process stopped; automatic restart cancelled"
        },
    );
    Ok(())
}

pub fn delete(state: &EngineState, id: &str) -> Result<()> {
    let supervisor = state.supervisor.lock().unwrap_or_else(|e| e.into_inner());
    let record = load(state, id)?;
    if record.pid.is_some()
        || record.desired_running
        || supervisor.is_managing(&record.profile.process_id())
    {
        bail!("stop the agent before deleting it");
    }
    state.repository.remove_agent(id)?;
    journal(
        state,
        &record,
        EventKind::AgentDeleted,
        EventSeverity::Info,
        "profile deleted",
    );
    Ok(())
}

/// Release a stale database identity without ever signalling an OS process.
/// This is separate from Stop so the operator explicitly chooses recovery.
pub fn forget_stale(state: &EngineState, id: &str) -> Result<()> {
    let supervisor = state.supervisor.lock().unwrap_or_else(|e| e.into_inner());
    let mut record = load(state, id)?;
    if supervisor.is_managing(&record.profile.process_id()) {
        bail!("stop the managed agent before clearing a stale PID");
    }
    if record.pid.is_none() {
        bail!("agent has no recorded PID to clear");
    }
    if identity_status(&record) == RecordedProcess::Alive {
        bail!("recorded agent is still alive; stop it before clearing its PID");
    }
    record.status = AgentStatus::Stopped;
    record.pid = None;
    record.process_started_at = None;
    record.desired_running = false;
    record.restart_after = None;
    record.restart_attempts = 0;
    record.healthy = None;
    state.repository.put_agent(&record)?;
    journal(state, &record, EventKind::AgentStopped, EventSeverity::Warning,
        "operator cleared a stale PID reference; no process was signalled and automatic restart is disabled");
    Ok(())
}

pub(super) fn schedule(record: &mut AgentRecord) {
    record.restart_after = None;
    if record.desired_running && record.profile.auto_restart && record.restart_attempts < 3 {
        record.restart_attempts += 1;
        record.restart_after = Some(now() + 5 * 2u64.pow(record.restart_attempts - 1));
    } else {
        record.desired_running = false;
    }
}

pub(super) fn journal(
    state: &EngineState,
    agent: &AgentRecord,
    kind: EventKind,
    severity: EventSeverity,
    message: &str,
) {
    let _ = state.repository.record_event(NewRuntimeEvent {
        node_id: agent.profile.node_id.clone(),
        node_name: Some(agent.profile.name.clone()),
        kind,
        severity,
        message: format!(
            "Agent {}: {}",
            agent.profile.name,
            crate::redaction::redact_sensitive_text(message)
        ),
    });
}

pub(super) fn log_path(state: &EngineState, id: &str) -> std::path::PathBuf {
    state.data_dir.join("logs").join(format!("agent-{id}.log"))
}

pub(super) fn identity_status(record: &AgentRecord) -> RecordedProcess {
    let result = recorded_process(&record.profile.identity(record.pid));
    if result == RecordedProcess::Alive
        && (record.process_started_at.is_none()
            || record.pid.and_then(process_started_at) != record.process_started_at)
    {
        return RecordedProcess::Reused;
    }
    result
}
