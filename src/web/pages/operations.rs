//! Fleet readiness: one focused view of launch and operational findings.

use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::core::operations::evaluate_fleet;
use crate::repository::Repository;

use super::super::{html, WebState};

pub async fn operations(State(state): State<WebState>) -> Response {
    match render(&state.repository) {
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

fn render(repository: &Repository) -> anyhow::Result<String> {
    let nodes = repository.list_nodes()?;
    let plugin_states = nodes
        .iter()
        .map(|node| {
            repository
                .list_plugin_states(&node.id)
                .map(|states| (node.id.clone(), states))
        })
        .collect::<anyhow::Result<std::collections::BTreeMap<_, _>>>()?;
    let diagnostics = evaluate_fleet(&nodes, &plugin_states);
    let readiness = diagnostics_text(&diagnostics);

    Ok(format!(
        r#"{head}
{stats}
<section class="surface"><div class="section-head"><h2>Fleet findings</h2><a href="/events">Open event journal</a></div>
<pre>{readiness}</pre></section>"#,
        head = html::page_head(
            "Readiness",
            "Validate launch configuration and surface the fleet findings that need operator attention.",
            r#"<a class="btn" href="/events">Events</a>"#,
        ),
        stats = html::cards(&[
            ("Fleet score", diagnostics.score.to_string()),
            ("Ready", diagnostics.ready_nodes.to_string()),
            ("Warnings", diagnostics.warning_count.to_string()),
            ("Critical", diagnostics.critical_count.to_string()),
        ]),
        readiness = html::escape(readiness.trim_end()),
    ))
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
