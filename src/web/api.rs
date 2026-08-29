//! JSON API for the polling script and scripted operators. Read endpoints
//! mirror the pages; control endpoints are intentionally page-only (form posts
//! with session cookies), so curl users keep the headless `--node-start`-style
//! CLI commands.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::core::workspace_queries;

use super::{fleet::Fleet, pages::metrics_page, WebState};

#[derive(Serialize)]
pub struct FleetNode {
    pub id: String,
    pub name: String,
    pub status: String,
    pub network: String,
    pub rpc_port: u16,
    pub rpc_health: String,
}

#[derive(Serialize)]
pub struct FleetPayload {
    pub nodes: Vec<FleetNode>,
}

pub async fn fleet(State(state): State<WebState>) -> Response {
    match Fleet::load(&state.repository) {
        Ok(fleet) => Json(FleetPayload {
            nodes: fleet
                .rows
                .iter()
                .map(|row| FleetNode {
                    id: row.node.id.clone(),
                    name: row.node.name.clone(),
                    status: row.node.status.label().to_string(),
                    network: row.node.network.to_string(),
                    rpc_port: row.node.rpc_port,
                    rpc_health: row.rpc_health.clone(),
                })
                .collect(),
        })
        .into_response(),
        Err(error) => error_response(&error),
    }
}

pub async fn readiness(State(state): State<WebState>) -> Response {
    let repository = &state.repository;
    let payload = (|| -> anyhow::Result<serde_json::Value> {
        let nodes = repository.list_nodes()?;
        let plugin_states = nodes
            .iter()
            .map(|node| {
                repository
                    .list_plugin_states(&node.id)
                    .map(|states| (node.id.clone(), states))
            })
            .collect::<anyhow::Result<std::collections::BTreeMap<_, _>>>()?;
        let diagnostics = crate::core::operations::evaluate_fleet(&nodes, &plugin_states);
        Ok(serde_json::json!({
            "score": diagnostics.score,
            "ready_nodes": diagnostics.ready_nodes,
            "warning_count": diagnostics.warning_count,
            "critical_count": diagnostics.critical_count,
            "events": workspace_queries::count_workspace_events(
                repository,
                &crate::events::RuntimeEventFilter::new(None, "", 1),
            )?,
        }))
    })();
    match payload {
        Ok(value) => Json(value).into_response(),
        Err(error) => error_response(&error),
    }
}

pub async fn metrics_prometheus(State(state): State<WebState>) -> Response {
    match metrics_page::collect_snapshot(&state.repository) {
        Ok(snapshot) => (
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; version=0.0.4",
            )],
            snapshot.to_prometheus_text(),
        )
            .into_response(),
        Err(error) => error_response(&error),
    }
}

fn error_response(error: &anyhow::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": error.to_string() })),
    )
        .into_response()
}
