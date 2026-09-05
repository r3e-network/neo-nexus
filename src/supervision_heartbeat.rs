//! What the outside world can see about the guardian itself.
//!
//! `/healthz` used to report only that the web process was alive: a guardian
//! thread that never started, or one stalled on a wedged probe, left the
//! endpoint green while nothing supervised the fleet. The heartbeat is the
//! fix — the loop stamps it every tick, a failed spawn marks it failed, and
//! the health endpoint turns it into a verdict.

use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

/// How long without a completed tick the guardian is reported stalled. The
/// bound has to exceed the worst legitimate tick — a full probe batch of
/// three-second timeouts — or a merely slow tick would cry wolf.
const STALL_AFTER: Duration = Duration::from_secs(90);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupervisionLiveness {
    NotRunning,
    Running,
    Stalled,
    Failed,
}

#[derive(Default)]
struct HeartbeatInner {
    state: Mutex<SupervisionStateInner>,
    failure: Mutex<Option<String>>,
}

#[derive(Default)]
struct SupervisionStateInner {
    running: bool,
    last_tick_unix: Option<u64>,
}

/// Shared handle between the guardian loop and whoever reports its health.
#[derive(Clone, Default)]
pub struct SupervisionHeartbeat {
    inner: Arc<HeartbeatInner>,
}

impl SupervisionHeartbeat {
    pub fn new() -> Self {
        Self::default()
    }

    /// The loop completed a tick.
    pub fn beat(&self) {
        self.beat_at(unix_now());
    }

    /// The same, at an explicit time, so staleness is testable.
    pub fn beat_at(&self, now_unix: u64) {
        let mut state = lock(&self.inner.state);
        state.running = true;
        state.last_tick_unix = Some(now_unix);
    }

    /// The guardian thread could not be started at all.
    pub fn mark_failed(&self, reason: String) {
        *lock(&self.inner.failure) = Some(reason);
    }

    /// The verdict /healthz reports, evaluated against wall-clock time.
    pub fn evaluate(&self) -> SupervisionReport {
        self.evaluate_at(unix_now())
    }

    /// The same verdict at an explicit time, so staleness is testable.
    pub fn evaluate_at(&self, now_unix: u64) -> SupervisionReport {
        let state = lock(&self.inner.state);
        if let Some(reason) = lock(&self.inner.failure).clone() {
            return SupervisionReport {
                liveness: SupervisionLiveness::Failed,
                detail: Some(reason),
            };
        }
        if !state.running {
            return SupervisionReport {
                liveness: SupervisionLiveness::NotRunning,
                detail: None,
            };
        }
        // A stale stamp means ticks stopped completing: the loop is wedged on
        // something, and whatever it last observed is aging into fiction.
        let age = state
            .last_tick_unix
            .map(|last| now_unix.saturating_sub(last))
            .unwrap_or_default();
        if age > STALL_AFTER.as_secs() {
            return SupervisionReport {
                liveness: SupervisionLiveness::Stalled,
                detail: Some(format!(
                    "no completed supervision tick for {age}s; the loop is likely blocked"
                )),
            };
        }
        SupervisionReport {
            liveness: SupervisionLiveness::Running,
            detail: None,
        }
    }
}

/// The verdict plus, for a stalled or failed guardian, why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupervisionReport {
    pub liveness: SupervisionLiveness,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "../tests/unit/supervision_heartbeat/tests.rs"]
mod tests;
