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
use log::error;
use serde::Serialize;

use super::{fleet::Fleet, WebState};

pub mod hermes_mcp;
pub mod iac;

pub use iac::{fleet_iac, node_iac};

/// Collect Prometheus-formatted metrics from all nodes.
/// Shared by both authenticated (/api/metrics-prometheus) and public (/public-metrics) routes.
pub fn collect_metrics_snapshot(
    workspace: &crate::core::workspace_queries::WorkspaceQueries,
) -> anyhow::Result<String> {
    let snapshot = crate::web::pages::metrics_page::collect_snapshot(workspace)?;
    Ok(snapshot.to_prometheus_text())
}

#[derive(Serialize)]
pub struct FleetNode {
    pub id: String,
    pub name: String,
    pub role: String,
    pub flavor: String,
    pub client: String,
    pub status: String,
    pub network: String,
    pub p2p_port: u16,
    pub rpc_port: u16,
    pub rpc_health: String,
    pub signer_binding: Option<String>,
    pub hermes_connected: bool,
}

#[derive(Serialize)]
pub struct FleetPayload {
    pub nodes: Vec<FleetNode>,
}

pub async fn fleet(State(state): State<WebState>) -> Response {
    let all_signers = state.workspace.list_all_signer_bindings().unwrap_or_default();
    let all_hermes = state.workspace.list_hermes_agents().unwrap_or_default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    match Fleet::load(&state.workspace) {
        Ok(fleet) => Json(FleetPayload {
            nodes: fleet
                .rows
                .iter()
                .map(|row| {
                    let role = state
                        .workspace
                        .load_node_role(&row.node.id)
                        .ok()
                        .flatten()
                        .map_or_else(|| "standard".to_string(), |r| r.slug().to_string());
                    let flavor = match role.as_str() {
                        "rpc-api" => "8 vCPU · 32 GB RAM · 1 Gbps",
                        "relay" => "2 vCPU · 4 GB RAM · Low Latency",
                        "validator" => "4 vCPU · 16 GB RAM · NVMe · Leased Signer",
                        "indexer" => "16 vCPU · 64 GB RAM · 2 TB NVMe",
                        "oracle" => "4 vCPU · 16 GB RAM · HTTPS Outbound",
                        "observer" => "2 vCPU · 8 GB RAM · 500 GB SSD",
                        _ => "General Purpose VM",
                    };
                    let signer = all_signers
                        .iter()
                        .find(|(nid, _)| nid == &row.node.id)
                        .map(|(_, b)| format!("{}/{}", b.backend_id, b.key_id));
                    let hermes_connected = all_hermes
                        .iter()
                        .find(|h| h.node_id == row.node.id)
                        .is_some_and(|h| h.is_alive(now));
                    FleetNode {
                        id: row.node.id.clone(),
                        name: row.node.name.clone(),
                        role,
                        flavor: flavor.to_string(),
                        client: row.node.node_type.to_string(),
                        status: row.node.status.label().to_string(),
                        network: row.node.network.to_string(),
                        p2p_port: row.node.p2p_port,
                        rpc_port: row.node.rpc_port,
                        rpc_health: row.rpc_health.clone(),
                        signer_binding: signer,
                        hermes_connected,
                    }
                })
                .collect(),
        })
        .into_response(),
        Err(error) => error_response(&error),
    }
}

pub async fn readiness(State(state): State<WebState>) -> Response {
    let repository = &state.workspace;
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
            "events": repository.count_events(
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
    match collect_metrics_snapshot(&state.workspace) {
        Ok(snapshot) => (
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; version=0.0.4",
            )],
            snapshot,
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

// ============================================================================
// Node-Specific Metrics Endpoint
// ============================================================================

#[derive(Serialize, Clone)]
pub struct NodeMetricsResponse {
    pub node_id: String,
    pub node_name: String,
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_progress: Option<SyncProgressResponse>,
}

#[derive(Serialize, Clone)]
pub struct SyncProgressResponse {
    pub current_height: u64,
    pub target_height: u64,
    pub sync_percentage: f32,
    pub peers_connected: u32,
}

pub async fn node_metrics(
    State(state): State<WebState>,
    axum::extract::Path(node_id): axum::extract::Path<String>,
) -> Response {
    match state.workspace.list_nodes() {
        Ok(nodes) => {
            let node = match nodes.iter().find(|n| n.id == node_id) {
                Some(n) => n,
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "error": format!("Node {} not found", node_id)
                        })),
                    )
                        .into_response()
                }
            };

            let supervisor = state.supervisor();
            let adapters = supervisor.adapters();

            let mut response = NodeMetricsResponse {
                node_id: node.id.clone(),
                node_name: node.name.clone(),
                pid: node.pid,
                metrics_endpoint: None,
                sync_progress: None,
            };

            if let Some(adapter) = adapters.get_metrics_adapter(&node.node_type) {
                if let Some(endpoint) = adapter.metrics_url(node.rpc_port) {
                    response.metrics_endpoint = Some(endpoint);
                }
            }

            // Read sync progress from the same-source observations snapshot the
            // collection worker publishes. This avoids re-reading the log file under
            // lock; an unknown value simply stays absent rather than being invented.
            if let Some(sync) = supervisor.observations().sync_progress(node) {
                response.sync_progress = Some(SyncProgressResponse {
                    current_height: sync.current_height,
                    target_height: sync.target_height,
                    sync_percentage: sync.sync_percentage,
                    peers_connected: sync.peers_connected,
                });
            }

            Json(response).into_response()
        }
        Err(error) => {
            error!("Failed to list nodes for metrics: {error:#}");
            error_response(&error)
        }
    }
}

// ============================================================================
// Plugin Inventory Endpoint
// ============================================================================

#[derive(serde::Deserialize)]
pub struct PluginsQuery {
    pub node_type: Option<String>,
}

#[derive(Serialize)]
pub struct PluginItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub compatible_node_types: Vec<String>,
    pub requires_restart: bool,
}

#[derive(Serialize)]
pub struct PluginsPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_type: Option<String>,
    pub plugins: Vec<PluginItem>,
}

pub async fn plugins(axum::extract::Query(query): axum::extract::Query<PluginsQuery>) -> Response {
    let catalog = crate::catalog::PluginCatalog;
    let (node_type_filter, definitions) = match query.node_type.as_deref() {
        Some(type_str) if !type_str.trim().is_empty() => {
            match <crate::types::NodeType as std::str::FromStr>::from_str(type_str.trim()) {
                Ok(nt) => (Some(nt.to_string()), catalog.for_node_type(nt)),
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "error": format!("unsupported node_type filter: {type_str}")
                        })),
                    )
                        .into_response();
                }
            }
        }
        _ => (None, catalog.all().iter().collect()),
    };

    let plugins = definitions
        .into_iter()
        .map(|def| PluginItem {
            id: def.id.to_string(),
            name: def.name.to_string(),
            category: def.category.to_string(),
            description: def.description.to_string(),
            compatible_node_types: def.node_types.iter().map(|nt| nt.to_string()).collect(),
            requires_restart: def.requires_restart,
        })
        .collect();

    Json(PluginsPayload {
        node_type: node_type_filter,
        plugins,
    })
    .into_response()
}
