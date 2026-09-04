use super::{
    lifecycle::{identity_status, journal, load, schedule, start_inner},
    profile::now,
    AgentStatus,
};
use crate::{
    events::{EventKind, EventSeverity},
    supervision::EngineState,
    supervisor::{ProcessExit, RecordedProcess},
};

pub fn observe_exit(state: &EngineState, exit: &ProcessExit) -> bool {
    let Some(id) = exit.process_id.strip_prefix("agent:") else {
        return false;
    };
    let _guard = state.supervisor.lock().unwrap_or_else(|e| e.into_inner());
    let Ok(mut agent) = load(state, id) else {
        return true;
    };
    // A delayed observation cannot overwrite a new process started by an operator.
    if agent.pid != Some(exit.pid) {
        return true;
    }
    agent.pid = None;
    agent.process_started_at = None;
    agent.healthy = None;
    let hermes = agent.profile.kind == super::AgentKind::Hermes;
    if hermes && exit.exit_code == Some(78) {
        agent.status = AgentStatus::Error;
        agent.desired_running = false;
        agent.restart_after = None;
    } else if hermes && exit.exit_code == Some(75) {
        agent.status = AgentStatus::Stopped;
        schedule(&mut agent);
    } else if exit.exit_code == Some(0) {
        agent.status = AgentStatus::Stopped;
        agent.desired_running = false;
        agent.restart_after = None;
    } else {
        agent.status = AgentStatus::Crashed;
        schedule(&mut agent);
    }
    if state.repository.put_agent(&agent).is_ok() {
        journal(
            state,
            &agent,
            EventKind::AgentExited,
            if exit.exit_code == Some(0) || hermes && exit.exit_code == Some(75) {
                EventSeverity::Info
            } else {
                EventSeverity::Critical
            },
            &format!(
                "process exited ({:?}); restart attempt {}, scheduled={}",
                exit.exit_code,
                agent.restart_attempts,
                agent.restart_after.is_some()
            ),
        );
    }
    true
}

pub fn tick(state: &EngineState) {
    let Ok(mut agents) = state.repository.list_agents() else {
        return;
    };
    let timestamp = now();
    for agent in &mut agents {
        {
            let supervisor = state.supervisor.lock().unwrap_or_else(|e| e.into_inner());
            // Re-read after acquiring the same lifecycle lock used by browser controls.
            let Ok(current) = load(state, &agent.profile.id) else {
                continue;
            };
            *agent = current;
            if agent.pid.is_some() && !supervisor.is_managing(&agent.profile.process_id()) {
                match identity_status(agent) {
                    RecordedProcess::Gone => {
                        agent.pid = None;
                        agent.process_started_at = None;
                        agent.status = AgentStatus::Crashed;
                        agent.healthy = None;
                        schedule(agent);
                        let _ = state.repository.put_agent(agent);
                        journal(
                            state,
                            agent,
                            EventKind::AgentExited,
                            EventSeverity::Critical,
                            "recorded process disappeared; exit code unavailable",
                        );
                    }
                    RecordedProcess::Reused => {
                        if agent.status != AgentStatus::Error {
                            agent.status = AgentStatus::Error;
                            agent.desired_running = false;
                            agent.restart_after = None;
                            let _ = state.repository.put_agent(agent);
                            journal(
                                state,
                                agent,
                                EventKind::AgentExited,
                                EventSeverity::Critical,
                                "PID identity mismatch; automatic restart blocked",
                            );
                        }
                    }
                    RecordedProcess::Alive => (),
                }
            }
        }
        if agent.pid.is_none()
            && agent.desired_running
            && agent.restart_after.is_some_and(|at| at <= timestamp)
        {
            let _ = start_inner(state, &agent.profile.id, false);
        }
    }
    // One HTTP check per tick, oldest first; lifecycle locks are never held over IO.
    agents.sort_by_key(|agent| agent.last_health_at);
    let Some(agent) = agents.into_iter().find(|agent| {
        agent.status == AgentStatus::Running
            && (agent.profile.health_url.is_some()
                || agent.profile.kind == super::AgentKind::Hermes)
            && timestamp.saturating_sub(agent.last_health_at) >= 30
    }) else {
        return;
    };
    let endpoint = agent.profile.health_url.as_deref().unwrap_or_default();
    let healthy = if agent.profile.kind == super::AgentKind::Hermes {
        super::hermes_runtime::runtime_health(
            &agent.profile,
            agent.pid.unwrap_or_default(),
            timestamp,
        )
        .unwrap_or(None)
    } else {
        Some(
            ureq::AgentBuilder::new()
                .redirects(0)
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .get(endpoint)
                .call()
                .is_ok_and(|response| (200..300).contains(&response.status())),
        )
    };
    let _guard = state.supervisor.lock().unwrap_or_else(|e| e.into_inner());
    let Ok(mut current) = load(state, &agent.profile.id) else {
        return;
    };
    if current.pid != agent.pid || current.status != AgentStatus::Running {
        return;
    }
    let changed = current.healthy != healthy;
    current.healthy = healthy;
    current.last_health_at = timestamp;
    if state.repository.put_agent(&current).is_ok() && changed && healthy.is_some() {
        journal(
            state,
            &current,
            EventKind::AgentHealthChanged,
            if healthy == Some(true) {
                EventSeverity::Info
            } else {
                EventSeverity::Warning
            },
            if healthy == Some(true) {
                "runtime health check passed"
            } else {
                "runtime health check failed"
            },
        );
    }
}
