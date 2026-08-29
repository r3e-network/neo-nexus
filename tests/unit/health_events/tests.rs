//! The mapping from a status to what an operator reads. Small, but it is the
//! difference between a journal that shows transitions and one that buries them.

use crate::{
    events::EventSeverity,
    federation::RemoteProbeStatus,
    health_events::{
        exit_notice, exit_was_clean, remote_probe_event_severity, rpc_health_event_severity,
        should_record_remote_probe_event, should_record_rpc_health_event,
    },
    rpc_health::RpcHealthStatus,
    supervisor::ProcessExit,
};

fn exit_with(code: Option<i32>) -> ProcessExit {
    ProcessExit {
        process_id: "node-1".to_string(),
        node_id: "node-1".to_string(),
        pid: 4242,
        exit_code: code,
    }
}

#[test]
fn a_clean_exit_is_worded_differently_from_a_failure() {
    assert_eq!(
        exit_notice("seed-1", &exit_with(Some(0))),
        "seed-1 exited normally"
    );
    assert_eq!(
        exit_notice("seed-1", &exit_with(Some(1))),
        "seed-1 exited with code 1"
    );
    // A signal leaves no code behind, which is not the same as code 0.
    assert_eq!(
        exit_notice("seed-1", &exit_with(None)),
        "seed-1 exited without a code"
    );
}

#[test]
fn only_an_explicit_zero_counts_as_clean() {
    assert!(exit_was_clean(&exit_with(Some(0))));
    assert!(!exit_was_clean(&exit_with(Some(1))));
    assert!(!exit_was_clean(&exit_with(None)));
}

#[test]
fn unreachable_rpc_is_critical_but_a_healthy_one_is_not_an_alarm() {
    assert_eq!(
        rpc_health_event_severity(RpcHealthStatus::Healthy),
        EventSeverity::Info
    );
    assert_eq!(
        rpc_health_event_severity(RpcHealthStatus::Degraded),
        EventSeverity::Warning
    );
    assert_eq!(
        rpc_health_event_severity(RpcHealthStatus::Unreachable),
        EventSeverity::Critical
    );
}

#[test]
fn a_federation_peer_that_cannot_be_reached_outranks_one_that_is_disabled() {
    assert_eq!(
        remote_probe_event_severity(RemoteProbeStatus::Healthy),
        EventSeverity::Info
    );
    assert_eq!(
        remote_probe_event_severity(RemoteProbeStatus::Disabled),
        EventSeverity::Warning
    );
    assert_eq!(
        remote_probe_event_severity(RemoteProbeStatus::Unreachable),
        EventSeverity::Critical
    );
}

#[test]
fn an_unchanged_status_is_not_journal_worthy() {
    assert!(!should_record_rpc_health_event(
        Some(RpcHealthStatus::Healthy),
        RpcHealthStatus::Healthy,
    ));
    assert!(should_record_rpc_health_event(
        Some(RpcHealthStatus::Healthy),
        RpcHealthStatus::Unreachable,
    ));
    // The first probe has no previous, so it is always worth one entry.
    assert!(should_record_rpc_health_event(
        None,
        RpcHealthStatus::Healthy
    ));

    assert!(!should_record_remote_probe_event(
        Some(RemoteProbeStatus::Degraded),
        RemoteProbeStatus::Degraded,
    ));
    assert!(should_record_remote_probe_event(
        Some(RemoteProbeStatus::Degraded),
        RemoteProbeStatus::Healthy,
    ));
}
