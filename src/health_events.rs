//! What an observation means: how severe it is, how to say it, and whether it
//! is news at all.
//!
//! These helpers lived inside `src/app/` beside the desktop shell and were
//! removed with it, though nothing in them is about drawing a window — each one
//! maps a status to a severity or a sentence. The background engine in
//! [`crate::supervision`] needs them to decide what it just saw is worth
//! writing down, so they live in the core where any frontend can reach them.

use crate::{
    events::EventSeverity,
    federation::RemoteProbeStatus,
    rpc_health::{RpcHealthReport, RpcHealthStatus},
    supervisor::ProcessExit,
};

/// An exit code of 0 means the node was told to stop. Anything else — including
/// no code at all, which is what a signal leaves behind — is a failure an
/// operator wants stated plainly.
pub fn exit_notice(node_name: &str, exit: &ProcessExit) -> String {
    match exit.exit_code {
        Some(0) => format!("{node_name} exited normally"),
        Some(code) => format!("{node_name} exited with code {code}"),
        None => format!("{node_name} exited without a code"),
    }
}

/// Whether an exit is the kind that should try to come back.
pub fn exit_was_clean(exit: &ProcessExit) -> bool {
    exit.exit_code == Some(0)
}

pub fn rpc_health_notice(report: &RpcHealthReport) -> String {
    format!("RPC health {}: {}", report.status_label(), report.message())
}

pub fn remote_probe_notice(
    profile_name: &str,
    report_status: RemoteProbeStatus,
    message: &str,
) -> String {
    format!(
        "federation {profile_name} is {}: {message}",
        report_status.label()
    )
}

pub fn rpc_health_event_severity(status: RpcHealthStatus) -> EventSeverity {
    match status {
        RpcHealthStatus::Healthy => EventSeverity::Info,
        RpcHealthStatus::Degraded => EventSeverity::Warning,
        RpcHealthStatus::Unreachable => EventSeverity::Critical,
    }
}

pub fn remote_probe_event_severity(status: RemoteProbeStatus) -> EventSeverity {
    match status {
        RemoteProbeStatus::Healthy => EventSeverity::Info,
        RemoteProbeStatus::Degraded | RemoteProbeStatus::Disabled => EventSeverity::Warning,
        RemoteProbeStatus::Unreachable => EventSeverity::Critical,
    }
}

/// A probe that says the same thing as the last one is not news. Recording every
/// successful tick would bury the transitions an operator actually reads.
pub fn should_record_rpc_health_event(
    previous: Option<RpcHealthStatus>,
    current: RpcHealthStatus,
) -> bool {
    previous != Some(current)
}

pub fn should_record_remote_probe_event(
    previous: Option<RemoteProbeStatus>,
    current: RemoteProbeStatus,
) -> bool {
    previous != Some(current)
}

#[cfg(test)]
#[path = "../tests/unit/health_events/tests.rs"]
mod tests;
