//! The remote federation probe the Settings page configures: rate-limited by
//! its own policy interval and journalling only on a change of status, so a
//! healthy server does not fill the journal with "still healthy".
//!
//! This node's own RPC endpoint used to be probed here too, one node per tick.
//! That probe now lives in [`super::observation`], which samples the fleet on
//! its own thread and writes the same `rpc_health_checks` rows from a richer
//! round. What stays here is the per-node retention the two share.

use std::time::{Duration, Instant};

use log::warn;

use crate::{
    events::{EventKind, NewRuntimeEvent},
    federation::RemoteFederationClient,
    health_events::{
        remote_probe_event_severity, remote_probe_notice, should_record_remote_probe_event,
    },
};

use super::state::{EngineState, LoopState};

const FEDERATION_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const RPC_HEALTH_RETAIN_PER_NODE: usize = 24;

impl LoopState {
    /// Whether something last done at `seen` is due again. Never having done it
    /// counts as due.
    fn due(&self, seen: Option<Instant>, now: Instant, interval: Duration) -> bool {
        seen.is_none_or(|seen| now.duration_since(seen) >= interval)
    }

    pub(super) fn probe_federation(&mut self, state: &EngineState) {
        let Ok(policy) = state.repository.load_remote_federation_monitor_policy() else {
            return;
        };
        if !policy.enabled {
            return;
        }
        let interval = policy.interval_duration();
        let now = Instant::now();
        let Ok(profiles) = state.repository.list_remote_servers() else {
            return;
        };
        let Some(profile) = profiles.into_iter().find(|profile| {
            profile.enabled
                && self.due(
                    self.federation_last_probe.get(&profile.id).copied(),
                    now,
                    interval,
                )
        }) else {
            return;
        };
        self.federation_last_probe.insert(profile.id.clone(), now);

        let report = match RemoteFederationClient::probe(&profile, FEDERATION_TIMEOUT) {
            Ok(report) => report,
            Err(error) => {
                warn!(
                    "neo-nexus: federation probe for {} failed: {error}",
                    profile.name
                );
                return;
            }
        };
        let previous = state
            .repository
            .latest_remote_server_probe(&profile.id)
            .ok()
            .flatten()
            .map(|record| record.status);
        if state
            .repository
            .record_remote_server_probe(&report)
            .is_err()
        {
            return;
        }
        if should_record_remote_probe_event(previous, report.status) {
            let message = remote_probe_notice(&profile.name, report.status, &report.message);
            let _ = state.repository.record_event(NewRuntimeEvent {
                node_id: None,
                node_name: None,
                kind: EventKind::RemoteServerProbed,
                severity: remote_probe_event_severity(report.status),
                message,
            });
        }
    }
}
