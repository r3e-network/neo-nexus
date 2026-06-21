use std::path::Path;

use crate::core::operations::FleetDiagnostics;

use super::{findings::workspace_readiness_findings, status::workspace_readiness_status};

pub(in crate::cli) fn workspace_readiness_text(
    db_path: &Path,
    diagnostics: &FleetDiagnostics,
) -> String {
    let mut lines = workspace_readiness_summary_lines(db_path, diagnostics);
    push_readiness_findings(&mut lines, diagnostics);
    lines.push(String::new());
    lines.join("\n")
}

fn workspace_readiness_summary_lines(
    db_path: &Path,
    diagnostics: &FleetDiagnostics,
) -> Vec<String> {
    vec![
        format!(
            "workspace-readiness: {}",
            workspace_readiness_status(diagnostics)
        ),
        format!("database: {}", db_path.display()),
        format!("score: {}", diagnostics.score),
        format!(
            "nodes: {}/{}",
            diagnostics.ready_nodes,
            diagnostics.nodes.len()
        ),
        format!("critical: {}", diagnostics.critical_count),
        format!("warnings: {}", diagnostics.warning_count),
    ]
}

fn push_readiness_findings(lines: &mut Vec<String>, diagnostics: &FleetDiagnostics) {
    let findings = workspace_readiness_findings(diagnostics);
    if findings.is_empty() {
        lines.push("finding: none".to_string());
        return;
    }

    lines.extend(findings.into_iter().map(|finding| {
        format!(
            "finding: {} | {} | {} | {} | resolve: {} | next: {}",
            finding.severity,
            finding.node_name,
            finding.title,
            finding.detail,
            finding.resolution_action,
            finding.resolution_hint
        )
    }));
}
