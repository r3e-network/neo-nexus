use crate::core::{operations::FleetDiagnostics, workspace::readiness_status};

pub(super) fn workspace_readiness_status(diagnostics: &FleetDiagnostics) -> &'static str {
    readiness_status(diagnostics)
}

pub(in crate::cli) fn workspace_readiness_exit_code(diagnostics: &FleetDiagnostics) -> i32 {
    if diagnostics.critical_count == 0 {
        0
    } else {
        1
    }
}
