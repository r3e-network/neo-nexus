use crate::{
    diagnostics::FleetDiagnostics, event_journal_report::EventJournalReport,
    metrics::MetricsSnapshot, readiness_report::readiness_status,
    workspace_integrity::WorkspaceIntegrityReport,
};

use super::{super::SupportBundleLogDiagnosisReport, status::bundle_status};
use crate::support_bundle::PRIVACY_POLICY;

pub(in crate::support_bundle) fn render_readme(
    diagnostics: &FleetDiagnostics,
    integrity: &WorkspaceIntegrityReport,
    events: &EventJournalReport,
    log_diagnosis: &SupportBundleLogDiagnosisReport,
    metrics: &MetricsSnapshot,
) -> String {
    format!(
        "NeoNexus support bundle\n\nstatus: {status}\nreadiness: {readiness}\nscore: {score}\nintegrity: {integrity}\nmetrics: {metrics_status}\nnode-processes: {node_processes}\nmissing-processes: {missing_processes}\nnodes: {nodes}\ncritical: {critical}\nwarnings: {warnings}\nlog-diagnoses: {log_diagnoses}\nlog-critical: {log_critical}\nlog-warnings: {log_warnings}\nmatched-events: {matched}\nexported-events: {exported}\nprivacy: {privacy}\n\nThis bundle is diagnostics evidence, not a workspace backup. It intentionally excludes private keys, wallet passwords, passphrases, mnemonics, seeds, bearer tokens, API keys, webhook secrets, raw logs, and raw database contents.\n",
        status = bundle_status(diagnostics, integrity, log_diagnosis, metrics),
        readiness = readiness_status(diagnostics),
        score = diagnostics.score,
        integrity = integrity.status_label(),
        metrics_status = metrics.status_label(),
        node_processes = metrics.node_processes.len(),
        missing_processes = metrics.missing_processes.len(),
        nodes = diagnostics.nodes.len(),
        critical = diagnostics.critical_count,
        warnings = diagnostics.warning_count,
        log_diagnoses = log_diagnosis.entries.len(),
        log_critical = log_diagnosis.critical_count,
        log_warnings = log_diagnosis.warning_count,
        matched = events.matched_event_count,
        exported = events.exported_event_count,
        privacy = PRIVACY_POLICY,
    )
}
