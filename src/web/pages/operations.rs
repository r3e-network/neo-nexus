//! Operations: the fleet readiness evaluation and the runtime event journal.

use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::core::{operations::evaluate_fleet, workspace_queries};
use crate::repository::Repository;

use super::super::{html, WebState};

pub async fn operations(State(state): State<WebState>) -> Response {
    match render(&state.repository) {
        Ok(body) => Html(html::layout("Operations", "operations", "", &body)).into_response(),
        Err(error) => Html(html::layout(
            "Operations",
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

    let filter = crate::events::RuntimeEventFilter::new(None, "", 50);
    let events = workspace_queries::list_workspace_events(repository, filter)?;
    let journal = if events.is_empty() {
        r#"<p class="muted">No events recorded yet.</p>"#.to_string()
    } else {
        let rows = events
            .iter()
            .map(|event| {
                format!(
                    r#"<tr><td>{}</td><td>{}</td><td>{}</td><td class="muted">{}</td></tr>"#,
                    event.occurred_at_unix,
                    html::escape(event.severity.label()),
                    html::escape(event.kind.label()),
                    html::escape(&event.message),
                )
            })
            .collect::<String>();
        format!(
            r#"<table><tr><th>Time (unix)</th><th>Severity</th><th>Kind</th><th>Message</th></tr>{rows}</table>"#
        )
    };

    Ok(format!(
        r#"<h1>Operations</h1>
<h2>Readiness</h2>
<pre>{readiness}</pre>
<h2>Event journal (latest 50)</h2>
{journal}"#,
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
