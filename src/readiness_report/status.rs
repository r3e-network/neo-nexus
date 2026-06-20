use crate::diagnostics::{FleetDiagnostics, NodeDiagnostics};

pub fn readiness_status(diagnostics: &FleetDiagnostics) -> &'static str {
    if diagnostics.critical_count > 0 {
        "blocked"
    } else if diagnostics.warning_count > 0 {
        "review"
    } else {
        "ready"
    }
}

pub(in crate::readiness_report) fn node_status(node: &NodeDiagnostics) -> &'static str {
    if node.critical_count() > 0 {
        "blocked"
    } else if node.warning_count() > 0 {
        "review"
    } else {
        "ready"
    }
}
