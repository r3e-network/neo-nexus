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
    let fields = [
        (
            "status",
            bundle_status(diagnostics, integrity, log_diagnosis, metrics).to_string(),
        ),
        ("readiness", readiness_status(diagnostics).to_string()),
        ("score", diagnostics.score.to_string()),
        ("integrity", integrity.status_label().to_string()),
        ("metrics", metrics.status_label().to_string()),
        ("node-processes", metrics.node_processes.len().to_string()),
        (
            "missing-processes",
            metrics.missing_processes.len().to_string(),
        ),
        ("nodes", diagnostics.nodes.len().to_string()),
        ("critical", diagnostics.critical_count.to_string()),
        ("warnings", diagnostics.warning_count.to_string()),
        ("log-diagnoses", log_diagnosis.entries.len().to_string()),
        ("log-critical", log_diagnosis.critical_count.to_string()),
        ("log-warnings", log_diagnosis.warning_count.to_string()),
        ("matched-events", events.matched_event_count.to_string()),
        ("exported-events", events.exported_event_count.to_string()),
        ("privacy", PRIVACY_POLICY.to_string()),
    ];

    let mut lines = vec!["NeoNexus support bundle".to_string(), String::new()];
    lines.extend(fields.map(|(label, value)| format!("{label}: {value}")));
    lines.push(String::new());
    lines.push(SUPPORT_BUNDLE_NOTICE.to_string());
    lines.push(String::new());
    lines.join("\n")
}

const SUPPORT_BUNDLE_NOTICE: &str = concat!(
    "This bundle is diagnostics evidence, not a workspace backup. ",
    "It intentionally excludes private keys, wallet passwords, passphrases, ",
    "mnemonics, seeds, bearer tokens, API keys, webhook secrets, raw logs, ",
    "and raw database contents."
);
