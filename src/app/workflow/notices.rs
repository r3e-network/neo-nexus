use crate::app::domain::{
    EventSeverity, ProcessExit, RpcHealthReport, RuntimeBinaryPreflight, RuntimeSmokeReport,
    RuntimeSmokeStatus,
};

pub(in crate::app) fn exit_notice(node_name: &str, exit: &ProcessExit) -> String {
    match exit.exit_code {
        Some(0) => format!("{node_name} exited normally"),
        Some(code) => format!("{node_name} exited with code {code}"),
        None => format!("{node_name} exited without a code"),
    }
}

pub(in crate::app) fn preflight_notice(report: &RuntimeBinaryPreflight) -> String {
    format!(
        "Binary preflight {}: {}",
        report.status_label(),
        report.operator_summary()
    )
}

pub(in crate::app) fn runtime_smoke_notice(report: &RuntimeSmokeReport) -> String {
    format!(
        "Runtime smoke {}: {}",
        report.status_label(),
        report.message
    )
}

pub(in crate::app) fn runtime_smoke_event_severity(status: RuntimeSmokeStatus) -> EventSeverity {
    match status {
        RuntimeSmokeStatus::Passed | RuntimeSmokeStatus::Review => EventSeverity::Info,
        RuntimeSmokeStatus::Failed | RuntimeSmokeStatus::TimedOut => EventSeverity::Warning,
        RuntimeSmokeStatus::Blocked => EventSeverity::Critical,
    }
}

pub(in crate::app) fn rpc_health_notice(report: &RpcHealthReport) -> String {
    format!("RPC health {}: {}", report.status_label(), report.message())
}
