use crate::{
    diagnostics::FleetDiagnostics, metrics::MetricsSnapshot,
    workspace_integrity::WorkspaceIntegrityReport,
};

use super::super::SupportBundleLogDiagnosisReport;

pub(in crate::support_bundle) fn bundle_status(
    diagnostics: &FleetDiagnostics,
    integrity: &WorkspaceIntegrityReport,
    log_diagnosis: &SupportBundleLogDiagnosisReport,
    metrics: &MetricsSnapshot,
) -> &'static str {
    if !integrity.is_success() {
        "failed"
    } else if diagnostics.critical_count > 0 {
        "blocked"
    } else if diagnostics.warning_count > 0
        || log_diagnosis.critical_count > 0
        || log_diagnosis.warning_count > 0
        || !metrics.is_success()
    {
        "review"
    } else {
        "ok"
    }
}
