use serde::Serialize;

use crate::diagnostics::{CheckSeverity, FleetDiagnostics};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct WorkspaceReadinessCliFinding {
    pub(super) severity: &'static str,
    pub(super) node_id: String,
    pub(super) node_name: String,
    pub(super) title: &'static str,
    pub(super) detail: String,
    pub(super) resolution_key: &'static str,
    pub(super) resolution: &'static str,
    pub(super) resolution_action: &'static str,
    pub(super) resolution_hint: &'static str,
}

pub(super) fn workspace_readiness_findings(
    diagnostics: &FleetDiagnostics,
) -> Vec<WorkspaceReadinessCliFinding> {
    [CheckSeverity::Critical, CheckSeverity::Warning]
        .into_iter()
        .flat_map(|severity| severity_findings(diagnostics, severity))
        .collect()
}

fn severity_findings(
    diagnostics: &FleetDiagnostics,
    severity: CheckSeverity,
) -> impl Iterator<Item = WorkspaceReadinessCliFinding> + '_ {
    diagnostics.nodes.iter().flat_map(move |node| {
        node.checks
            .iter()
            .filter(move |check| check.severity == severity)
            .map(move |check| WorkspaceReadinessCliFinding {
                severity: severity.label(),
                node_id: node.node_id.clone(),
                node_name: node.node_name.clone(),
                title: check.title,
                detail: check.detail.clone(),
                resolution_key: check.resolution.key(),
                resolution: check.resolution.label(),
                resolution_action: check.resolution.action_label(),
                resolution_hint: check.resolution.hint(),
            })
    })
}
