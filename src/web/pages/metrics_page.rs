//! Metrics: the workspace metrics snapshot as text, plus the Prometheus
//! exposition the release pipeline already consumes.
//!
//! Every reading comes from the server's one long-lived [`MetricsStore`]. This
//! page used to build its own `MetricsCollector::new(Duration::ZERO)` and
//! refresh it immediately, which is two `sysinfo` samples inside the 200 ms
//! minimum CPU interval — so its CPU figure was the constructor's first
//! reading, every time, on every surface that did the same.

use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::metrics::MetricsSnapshot;

use super::super::{html, WebState};

pub async fn metrics(State(state): State<WebState>) -> Response {
    match render(&state) {
        Ok(body) => Html(html::layout("Metrics", "metrics", "", &body)).into_response(),
        Err(error) => Html(html::layout(
            "Metrics",
            "metrics",
            &format!("failed to collect metrics: {error}"),
            "",
        ))
        .into_response(),
    }
}

fn render(state: &WebState) -> anyhow::Result<String> {
    let snapshot = collect_snapshot(state)?;
    Ok(format!(
        r#"<h1>Metrics</h1>
<h2>Snapshot</h2>
<pre>{text}</pre>
<h2>Prometheus</h2>
<pre>{prom}</pre>
<p class="muted">Scrape the same exposition from <code>/api/metrics-prometheus</code>.</p>"#,
        text = html::escape(snapshot.to_cli_text().trim_end()),
        prom = html::escape(snapshot.to_prometheus_text().trim_end()),
    ))
}

/// The server's current reading, shared with the JSON API and every page.
pub fn collect_snapshot(state: &WebState) -> anyhow::Result<MetricsSnapshot> {
    let nodes = state.workspace.list_nodes()?;
    Ok(state.metrics().snapshot(&nodes))
}
