use super::*;

#[path = "policies/alerts.rs"]
mod alerts;

#[test]
fn rpc_health_event_is_recorded_only_for_first_or_changed_status() {
    assert!(should_record_rpc_health_event(
        None,
        RpcHealthStatus::Healthy
    ));
    assert!(!should_record_rpc_health_event(
        Some(RpcHealthStatus::Healthy),
        RpcHealthStatus::Healthy
    ));
    assert!(should_record_rpc_health_event(
        Some(RpcHealthStatus::Healthy),
        RpcHealthStatus::Degraded
    ));
    assert!(should_record_rpc_health_event(
        Some(RpcHealthStatus::Degraded),
        RpcHealthStatus::Unreachable
    ));
}

#[test]
fn rpc_health_monitor_policy_draft_validates_and_normalizes_interval() {
    let policy = RpcHealthMonitorPolicy {
        enabled: false,
        interval_seconds: 120,
    };
    let draft = RpcHealthMonitorPolicyDraft::from_policy(policy);

    assert!(draft.validation_message().is_none());
    assert!(!draft.differs_from(policy));
    assert_eq!(draft.to_policy(), policy);

    let too_short = RpcHealthMonitorPolicyDraft {
        enabled: true,
        interval_seconds: 1,
    };
    assert!(too_short.validation_message().is_some());
    assert_eq!(
        too_short.to_policy().interval_seconds,
        RpcHealthMonitorPolicy::MIN_INTERVAL_SECONDS
    );
}

#[test]
fn remote_probe_event_is_recorded_only_for_first_or_changed_status() {
    assert!(should_record_remote_probe_event(
        None,
        RemoteProbeStatus::Healthy
    ));
    assert!(!should_record_remote_probe_event(
        Some(RemoteProbeStatus::Healthy),
        RemoteProbeStatus::Healthy
    ));
    assert!(should_record_remote_probe_event(
        Some(RemoteProbeStatus::Healthy),
        RemoteProbeStatus::Degraded
    ));
    assert!(should_record_remote_probe_event(
        Some(RemoteProbeStatus::Degraded),
        RemoteProbeStatus::Unreachable
    ));
}

#[test]
fn remote_federation_monitor_policy_draft_validates_and_normalizes_interval() {
    let policy = RemoteFederationMonitorPolicy {
        enabled: false,
        interval_seconds: 600,
    };
    let draft = RemoteFederationMonitorPolicyDraft::from_policy(policy);

    assert!(draft.validation_message().is_none());
    assert!(!draft.differs_from(policy));
    assert_eq!(draft.to_policy(), policy);

    let too_short = RemoteFederationMonitorPolicyDraft {
        enabled: true,
        interval_seconds: 1,
    };
    assert!(too_short.validation_message().is_some());
    assert_eq!(
        too_short.to_policy().interval_seconds,
        RemoteFederationMonitorPolicy::MIN_INTERVAL_SECONDS
    );
}
