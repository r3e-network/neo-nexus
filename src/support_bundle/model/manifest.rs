use serde::Serialize;

use crate::{
    diagnostics::FleetDiagnostics, metrics::MetricsSnapshot, readiness_report::readiness_status,
    workspace_integrity::WorkspaceIntegrityReport,
};

use super::{files::SupportBundleFile, logs::SupportBundleLogDiagnosisReport};
use crate::support_bundle::PRIVACY_POLICY;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceSupportBundleManifest {
    pub schema_version: u32,
    pub application: &'static str,
    pub application_version: String,
    pub generated_at_unix: u64,
    pub database: String,
    pub privacy_policy: &'static str,
    pub readiness_status: String,
    pub readiness_score: usize,
    pub integrity_status: String,
    pub node_count: usize,
    pub running_nodes: usize,
    pub warning_count: usize,
    pub critical_count: usize,
    pub metrics_status: String,
    pub node_process_count: usize,
    pub missing_process_count: usize,
    pub log_diagnosis_count: usize,
    pub log_diagnosis_warning_count: usize,
    pub log_diagnosis_critical_count: usize,
    pub matched_event_count: usize,
    pub exported_event_count: usize,
    pub files: Vec<SupportBundleFile>,
}

pub(in crate::support_bundle) struct WorkspaceSupportBundleManifestInput<'a> {
    pub(in crate::support_bundle) application_version: String,
    pub(in crate::support_bundle) generated_at_unix: u64,
    pub(in crate::support_bundle) database: String,
    pub(in crate::support_bundle) diagnostics: &'a FleetDiagnostics,
    pub(in crate::support_bundle) integrity: &'a WorkspaceIntegrityReport,
    pub(in crate::support_bundle) metrics: &'a MetricsSnapshot,
    pub(in crate::support_bundle) log_diagnosis: &'a SupportBundleLogDiagnosisReport,
    pub(in crate::support_bundle) running_nodes: usize,
    pub(in crate::support_bundle) matched_event_count: usize,
    pub(in crate::support_bundle) exported_event_count: usize,
    pub(in crate::support_bundle) files: Vec<SupportBundleFile>,
}

impl WorkspaceSupportBundleManifest {
    pub(in crate::support_bundle) fn from_input(
        input: WorkspaceSupportBundleManifestInput<'_>,
    ) -> Self {
        Self {
            schema_version: 1,
            application: "NeoNexus",
            application_version: input.application_version,
            generated_at_unix: input.generated_at_unix,
            database: input.database,
            privacy_policy: PRIVACY_POLICY,
            readiness_status: readiness_status(input.diagnostics).to_string(),
            readiness_score: input.diagnostics.score,
            integrity_status: input.integrity.status_label().to_string(),
            node_count: input.diagnostics.nodes.len(),
            running_nodes: input.running_nodes,
            warning_count: input.diagnostics.warning_count,
            critical_count: input.diagnostics.critical_count,
            metrics_status: input.metrics.status_label().to_string(),
            node_process_count: input.metrics.node_processes.len(),
            missing_process_count: input.metrics.missing_processes.len(),
            log_diagnosis_count: input.log_diagnosis.entries.len(),
            log_diagnosis_warning_count: input.log_diagnosis.warning_count,
            log_diagnosis_critical_count: input.log_diagnosis.critical_count,
            matched_event_count: input.matched_event_count,
            exported_event_count: input.exported_event_count,
            files: input.files,
        }
    }
}
