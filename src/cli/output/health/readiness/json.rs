use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::diagnostics::FleetDiagnostics;

use super::{
    findings::{workspace_readiness_findings, WorkspaceReadinessCliFinding},
    status::workspace_readiness_status,
};
use crate::cli::output::json_text;

#[derive(Debug, Serialize)]
struct WorkspaceReadinessJsonReport {
    schema_version: u32,
    status: &'static str,
    database: String,
    score: usize,
    node_count: usize,
    ready_nodes: usize,
    critical_count: usize,
    warning_count: usize,
    findings: Vec<WorkspaceReadinessCliFinding>,
}

pub(in crate::cli) fn workspace_readiness_json_text(
    db_path: &Path,
    diagnostics: &FleetDiagnostics,
) -> Result<String> {
    json_text(&WorkspaceReadinessJsonReport {
        schema_version: 1,
        status: workspace_readiness_status(diagnostics),
        database: db_path.display().to_string(),
        score: diagnostics.score,
        node_count: diagnostics.nodes.len(),
        ready_nodes: diagnostics.ready_nodes,
        critical_count: diagnostics.critical_count,
        warning_count: diagnostics.warning_count,
        findings: workspace_readiness_findings(diagnostics),
    })
}
