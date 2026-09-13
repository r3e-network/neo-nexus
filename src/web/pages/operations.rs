//! Fleet readiness: one focused view of launch and operational findings.

use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::core::operations::evaluate_fleet;
use crate::core::workspace_queries::WorkspaceQueries;

use super::super::{html, WebState};

pub async fn operations(State(state): State<WebState>) -> Response {
    match render(&state.workspace) {
        Ok(body) => Html(html::layout("Readiness", "operations", "", &body)).into_response(),
        Err(error) => Html(html::layout(
            "Readiness",
            "operations",
            &format!("failed to load operations data: {error}"),
            "",
        ))
        .into_response(),
    }
}

fn render(workspace: &WorkspaceQueries) -> anyhow::Result<String> {
    let nodes = workspace.list_nodes()?;
    let plugin_states = nodes
        .iter()
        .map(|node| {
            workspace
                .list_plugin_states(&node.id)
                .map(|states| (node.id.clone(), states))
        })
        .collect::<anyhow::Result<std::collections::BTreeMap<_, _>>>()?;
    let diagnostics = evaluate_fleet(&nodes, &plugin_states);
    let readiness = diagnostics_text(&diagnostics);

    let breadcrumb = html::breadcrumb(&[
        ("Systems Manager", "/operations"),
        ("OpsCenter", "/operations"),
        ("Operational findings", ""),
    ]);
    let head = html::page_head(
        "Systems Manager OpsCenter",
        "AWS Systems Manager OpsCenter: aggregate fleet readiness, operational findings (OpsItems), and SRE remediation automation.",
        r#"<a class="btn" href="/events">📜 CloudTrail Events</a> <a class="btn" href="/monitor">📊 CloudWatch Metrics</a> <a class="btn primary" href="/nodes">⬡ EC2 Instances</a>"#,
    );

    let ssm_bar = r#"<div class="aws-action-bar" style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; padding: 10px 14px; flex-wrap: wrap; gap: 8px;">
        <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
            <span style="font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--muted);">SSM Run Command:</span>
            <form method="post" action="/nodes/batch-action" style="display: inline; margin: 0;">
                <input type="hidden" name="action" value="smoke">
                <button class="btn small primary" type="submit" title="Run diagnostic sweep across all managed nodes">🩺 AWS-RunDiagnosticsSweep</button>
            </form>
            <form method="post" action="/nodes/batch-action" style="display: inline; margin: 0;">
                <input type="hidden" name="action" value="restart">
                <button class="btn small" type="submit" title="Supervised rolling restart of fleet">🔄 AWS-RestartNodes</button>
            </form>
            <a class="btn small" href="/api/fleet" download="fleet-inventory.json" title="Export complete fleet inventory">📥 AWS-InventoryExport</a>
        </div>
        <div class="muted mono" style="font-size: 11px;">
            Document: AWS-RunShellScript/NeoNexus-SREProbe
        </div>
    </div>"#;

    let opsitems_table = render_opsitems_table(&diagnostics);

    Ok(format!(
        r#"{breadcrumb}
{head}
{stats}
{ssm_bar}
<section class="surface">
    <div class="section-head">
        <h2>Operational Findings (OpsItems)</h2>
        <a href="/events">Open CloudTrail Journal</a>
    </div>
    {opsitems_table}
</section>
<details style="margin-top: 16px;">
    <summary style="cursor: pointer; font-size: 12px; color: var(--muted);">Raw Diagnostic Evidence</summary>
    <pre>{readiness}</pre>
</details>"#,
        breadcrumb = breadcrumb,
        head = head,
        stats = html::cards(&[
            ("Fleet score", format!("{}/100", diagnostics.score)),
            ("Ready instances", diagnostics.ready_nodes.to_string()),
            ("Warning findings", diagnostics.warning_count.to_string()),
            ("Critical findings", diagnostics.critical_count.to_string()),
        ]),
        ssm_bar = ssm_bar,
        opsitems_table = opsitems_table,
        readiness = html::escape(readiness.trim_end()),
    ))
}

fn render_opsitems_table(diagnostics: &crate::diagnostics::FleetDiagnostics) -> String {
    let mut rows = Vec::new();
    let mut count = 0;
    for node in &diagnostics.nodes {
        for check in &node.checks {
            if check.severity == crate::diagnostics::CheckSeverity::Critical
                || check.severity == crate::diagnostics::CheckSeverity::Warning
            {
                count += 1;
                let id_cell = format!(
                    r#"<span class="mono" style="font-size: 11px;">ops-{:03}</span>"#,
                    count
                );
                let node_cell = format!(
                    r#"<a href="/nodes/{}" style="font-weight: 600;">{}</a>"#,
                    html::urlencoding_lite(&node.node_id),
                    html::escape(&node.node_name)
                );
                let sev_badge = match check.severity {
                    crate::diagnostics::CheckSeverity::Critical => {
                        r#"<span class="badge error">▲ CRITICAL</span>"#
                    }
                    crate::diagnostics::CheckSeverity::Warning => {
                        r#"<span class="badge stopped">▲ WARNING</span>"#
                    }
                    _ => r#"<span class="badge">INFO</span>"#,
                };
                let action_cell = format!(
                    r#"<a class="btn small" href="/nodes/{}">{}</a>"#,
                    html::urlencoding_lite(&node.node_id),
                    check.resolution.action_label()
                );
                rows.push(html::row(&[
                    html::raw_cell(&id_cell),
                    html::raw_cell(&node_cell),
                    html::raw_cell(sev_badge),
                    html::cell(check.title),
                    html::cell(&check.detail),
                    html::raw_cell(r#"<span class="badge running">● OPEN</span>"#),
                    html::raw_cell(&action_cell),
                ]));
            }
        }
    }

    if rows.is_empty() {
        r#"<div class="notice ok" role="status" style="margin: 12px 0;"><strong>All Systems Operational</strong> — All status checks and operational configurations pass. No open OpsItems requiring remediation.</div>"#.to_string()
    } else {
        html::table(
            &[
                "OpsItem ID",
                "Resource",
                "Severity",
                "Title",
                "Finding Detail",
                "Status",
                "Remediation Action",
            ],
            &rows,
        )
    }
}

fn diagnostics_text(diagnostics: &crate::diagnostics::FleetDiagnostics) -> String {
    let mut lines = vec![format!(
        "fleet score {}/100 — {} nodes ready, {} warnings, {} critical findings",
        diagnostics.score,
        diagnostics.ready_nodes,
        diagnostics.warning_count,
        diagnostics.critical_count,
    )];
    for node in &diagnostics.nodes {
        if node.critical_count() > 0 || node.warning_count() > 0 {
            lines.push(format!(
                "{} — score {}, {} warnings, {} critical",
                node.node_name,
                node.score,
                node.warning_count(),
                node.critical_count(),
            ));
        }
    }
    if lines.len() == 1 {
        lines.push("every node is launch-ready".to_string());
    }
    lines.join("\n")
}
