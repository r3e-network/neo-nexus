//! Unattended custody health monitoring, independent of workbench page views.

use std::time::{Duration, Instant};

use crate::{
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    supervision::EngineState,
};

use super::SignerClient;

const PROBE_INTERVAL: Duration = Duration::from_secs(30);
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, PartialEq, Eq)]
enum HealthClass {
    Healthy,
    Degraded,
    Unreachable,
    InvalidConfiguration,
}

pub struct SignerMonitor {
    client: Result<Option<SignerClient>, String>,
    next_probe: Option<Instant>,
    last: Option<HealthClass>,
}

impl SignerMonitor {
    /// Resolve the same profile used by the signer workbench. An absent
    /// profile remains silent; an invalid one produces one actionable event.
    pub fn bootstrap() -> Self {
        Self {
            client: SignerClient::from_env("NEONEXUS_SIGNER_ADMIN")
                .map_err(|error| error.to_string()),
            next_probe: None,
            last: None,
        }
    }

    pub fn with_client(client: SignerClient) -> Self {
        Self {
            client: Ok(Some(client)),
            next_probe: None,
            last: None,
        }
    }

    pub fn tick(&mut self, state: &EngineState) {
        self.tick_at(state, Instant::now());
    }

    fn tick_at(&mut self, state: &EngineState, now: Instant) {
        if self.next_probe.is_some_and(|next| now < next) || matches!(&self.client, Ok(None)) {
            return;
        }
        self.next_probe = Some(now + PROBE_INTERVAL);
        let (observed, message) = match &self.client {
            Ok(Some(client)) => match client.health_with_timeout(PROBE_TIMEOUT) {
                Ok(health) if health.status.trim() == "ok" => (
                    HealthClass::Healthy,
                    format!(
                        "signer reports ok at {} (background probe)",
                        client.endpoint()
                    ),
                ),
                Ok(health) => (
                    HealthClass::Degraded,
                    format!(
                        "signer reports {} at {} (background probe)",
                        crate::redaction::redact_sensitive_text(&health.status),
                        client.endpoint()
                    ),
                ),
                Err(error) => (
                    HealthClass::Unreachable,
                    format!(
                        "signer health request failed at {} (background probe): {}",
                        client.endpoint(),
                        crate::redaction::redact_sensitive_text(&error.to_string())
                    ),
                ),
            },
            Err(error) => (
                HealthClass::InvalidConfiguration,
                format!(
                    "signer background monitoring configuration is invalid: {}",
                    crate::redaction::redact_sensitive_text(error)
                ),
            ),
            Ok(None) => return,
        };
        if self.last == Some(observed) {
            return;
        }
        let event = NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::SignerHealthChanged,
            severity: if observed == HealthClass::Healthy {
                EventSeverity::Info
            } else {
                EventSeverity::Warning
            },
            message,
        };
        // A database failure must not permanently consume a transition.
        if state.repository.record_event(event).is_ok() {
            self.last = Some(observed);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/signer_client/monitor.rs"]
mod tests;
