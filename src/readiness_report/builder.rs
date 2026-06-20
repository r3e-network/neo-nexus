use std::path::Path;

use crate::diagnostics::{CheckSeverity, DiagnosticCheck, FleetDiagnostics, NodeDiagnostics};

use super::{
    model::{
        WorkspaceReadinessCheckReport, WorkspaceReadinessFindingReport,
        WorkspaceReadinessNodeReport, WorkspaceReadinessReport,
    },
    status::{node_status, readiness_status},
};

impl WorkspaceReadinessReport {
    pub fn from_diagnostics(
        database: impl AsRef<Path>,
        diagnostics: &FleetDiagnostics,
        application_version: impl Into<String>,
        generated_at_unix: u64,
    ) -> Self {
        Self {
            schema_version: 1,
            application: "NeoNexus",
            application_version: application_version.into(),
            generated_at_unix,
            database: database.as_ref().display().to_string(),
            status: readiness_status(diagnostics),
            score: diagnostics.score,
            node_count: diagnostics.nodes.len(),
            ready_nodes: diagnostics.ready_nodes,
            critical_count: diagnostics.critical_count,
            warning_count: diagnostics.warning_count,
            findings: readiness_findings(diagnostics),
            nodes: readiness_nodes(diagnostics),
        }
    }
}

fn readiness_nodes(diagnostics: &FleetDiagnostics) -> Vec<WorkspaceReadinessNodeReport> {
    diagnostics.nodes.iter().map(readiness_node).collect()
}

fn readiness_node(node: &NodeDiagnostics) -> WorkspaceReadinessNodeReport {
    WorkspaceReadinessNodeReport {
        node_id: node.node_id.clone(),
        node_name: node.node_name.clone(),
        score: node.score,
        status: node_status(node),
        critical_count: node.critical_count(),
        warning_count: node.warning_count(),
        checks: node.checks.iter().map(readiness_check).collect(),
    }
}

fn readiness_check(check: &DiagnosticCheck) -> WorkspaceReadinessCheckReport {
    WorkspaceReadinessCheckReport {
        severity: check.severity.label(),
        title: check.title,
        detail: check.detail.clone(),
    }
}

fn readiness_findings(diagnostics: &FleetDiagnostics) -> Vec<WorkspaceReadinessFindingReport> {
    [CheckSeverity::Critical, CheckSeverity::Warning]
        .into_iter()
        .flat_map(|severity| findings_for_severity(diagnostics, severity))
        .collect()
}

fn findings_for_severity(
    diagnostics: &FleetDiagnostics,
    severity: CheckSeverity,
) -> Vec<WorkspaceReadinessFindingReport> {
    diagnostics
        .nodes
        .iter()
        .flat_map(|node| node_findings_for_severity(node, severity))
        .collect()
}

fn node_findings_for_severity(
    node: &NodeDiagnostics,
    severity: CheckSeverity,
) -> Vec<WorkspaceReadinessFindingReport> {
    node.checks
        .iter()
        .filter(|check| check.severity == severity)
        .map(|check| WorkspaceReadinessFindingReport {
            severity: severity.label(),
            node_id: node.node_id.clone(),
            node_name: node.node_name.clone(),
            title: check.title,
            detail: check.detail.clone(),
        })
        .collect()
}
