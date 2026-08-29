//! Metrics: the workspace metrics snapshot as text, plus the Prometheus
//! exposition the release pipeline already consumes.

use std::time::{Duration, Instant};

use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::metrics::{MetricsCollector, MetricsSnapshot};

use super::super::{html, WebState};

pub async fn metrics(State(state): State<WebState>) -> Response {
    match render(&state.repository) {
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

fn render(repository: &crate::repository::Repository) -> anyhow::Result<String> {
    let nodes = repository.list_nodes()?;
    let mut collector = MetricsCollector::new(Duration::ZERO);
    let snapshot = collector.refresh(&nodes, Instant::now());
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

/// Snapshot builder shared with the JSON API.
pub fn collect_snapshot(
    repository: &crate::repository::Repository,
) -> anyhow::Result<MetricsSnapshot> {
    let nodes = repository.list_nodes()?;
    let mut collector = MetricsCollector::new(Duration::ZERO);
    Ok(collector.refresh(&nodes, Instant::now()))
}
