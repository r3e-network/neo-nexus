//! The two health probes the Settings page configures: this node's RPC endpoint
//! and the remote federation servers. Both are rate-limited by their own policy
//! interval and both only journal on a change of status, so a healthy node does
//! not fill the journal with "still healthy".

use std::time::{Duration, Instant};

use log::warn;

use crate::{
    events::{EventKind, NewRuntimeEvent},
    federation::RemoteFederationClient,
    health_events::{
        remote_probe_event_severity, remote_probe_notice, rpc_health_event_severity,
        rpc_health_notice, should_record_remote_probe_event, should_record_rpc_health_event,
    },
    rpc_health::probe_node_rpc,
};

use super::state::{EngineState, LoopState};

const RPC_HEALTH_TIMEOUT: Duration = Duration::from_secs(3);
const FEDERATION_TIMEOUT: Duration = Duration::from_secs(5);
const RPC_HEALTH_RETAIN_PER_NODE: usize = 24;

impl LoopState {
    /// Whether something last done at `seen` is due again. Never having done it
    /// counts as due.
    fn due(&self, seen: Option<Instant>, now: Instant, interval: Duration) -> bool {
        seen.is_none_or(|seen| now.duration_since(seen) >= interval)
    }

    pub(super) fn probe_rpc_health(&mut self, state: &EngineState) {
        let Ok(policy) = state.repository.load_rpc_health_monitor_policy() else {
            return;
        };
        if !policy.enabled {
            return;
        }
        let interval = policy.interval_duration();
        let now = Instant::now();
        let Some(node) = state.nodes().into_iter().find(|node| {
            node.status.is_running()
                && node.rpc_port > 0
                && self.due(self.rpc_last_probe.get(&node.id).copied(), now, interval)
        }) else {
            return;
        };
        self.rpc_last_probe.insert(node.id.clone(), now);

        let report = probe_node_rpc(&node, RPC_HEALTH_TIMEOUT);
        let previous = state
            .repository
            .latest_rpc_health(&node.id)
            .ok()
            .flatten()
            .map(|record| record.status);
        if state.repository.record_rpc_health(&node, &report).is_err() {
            return;
        }
        let _ = state
            .repository
            .prune_rpc_health_keep_recent_per_node(RPC_HEALTH_RETAIN_PER_NODE);
        if should_record_rpc_health_event(previous, report.status) {
            let message = rpc_health_notice(&report);
            state.journal(
                &node,
                EventKind::RpcHealthChecked,
                rpc_health_event_severity(report.status),
                format!("Automatic RPC health: {message}"),
            );
        }
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
