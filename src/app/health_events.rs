use crate::app::domain::{
    EventSeverity, RemoteProbeStatus, RemoteServerProbeReport, RemoteServerProfile, RpcHealthStatus,
};

use super::current_unix_time;

pub(super) fn rpc_health_event_severity(status: RpcHealthStatus) -> EventSeverity {
    match status {
        RpcHealthStatus::Healthy => EventSeverity::Info,
        RpcHealthStatus::Degraded => EventSeverity::Warning,
        RpcHealthStatus::Unreachable => EventSeverity::Critical,
    }
}

pub(super) fn remote_probe_event_severity(status: RemoteProbeStatus) -> EventSeverity {
    match status {
        RemoteProbeStatus::Healthy => EventSeverity::Info,
        RemoteProbeStatus::Degraded | RemoteProbeStatus::Disabled => EventSeverity::Warning,
        RemoteProbeStatus::Unreachable => EventSeverity::Critical,
    }
}

pub(super) fn remote_probe_failure_report(
    profile: &RemoteServerProfile,
    error: &str,
) -> RemoteServerProbeReport {
    let message = format!(
        "Remote federation probe failed for {}: {error}",
        profile.name
    );
    RemoteServerProbeReport {
        profile_id: profile.id.clone(),
        profile_name: profile.name.clone(),
        base_url: profile.base_url.clone(),
        checked_at_unix: current_unix_time().unwrap_or_default(),
        status: RemoteProbeStatus::Unreachable,
        total_nodes: None,
        running_nodes: None,
        syncing_nodes: None,
        error_nodes: None,
        total_blocks: None,
        total_peers: None,
        public_node_count: None,
        message,
    }
}

pub(super) fn should_record_rpc_health_event(
    previous_status: Option<RpcHealthStatus>,
    current_status: RpcHealthStatus,
) -> bool {
    previous_status != Some(current_status)
}

pub(super) fn should_record_remote_probe_event(
    previous_status: Option<RemoteProbeStatus>,
    current_status: RemoteProbeStatus,
) -> bool {
    previous_status != Some(current_status)
}
