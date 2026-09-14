//! Minimal read-only federation API.
//!
//! Only aggregate node state is public. Node identities, runtime versions,
//! host/process metrics and per-node health stay behind the operator session.

use log::error;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::types::{NodeConfig, NodeStatus};

use super::WebState;

#[derive(Serialize)]
struct PublicStatusEnvelope {
    status: PublicStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicStatus {
    total_nodes: usize,
    running_nodes: usize,
    syncing_nodes: usize,
    error_nodes: usize,
    /// Detailed chain height and peer data are intentionally not published.
    total_blocks: Option<u64>,
    total_peers: Option<u64>,
    /// Nodes whose chain state needs someone. Published as a count only — which
    /// node and why is behind the session.
    nodes_needing_attention: usize,
    timestamp: u64,
}

pub async fn status(State(state): State<WebState>) -> Response {
    let payload = (|| -> anyhow::Result<PublicStatusEnvelope> {
        let nodes = state.workspace.list_nodes()?;
        // `syncing_nodes` counted `NodeStatus::Starting`, a state a node leaves
        // after a 600 ms launch settle window — under a column that federation
        // renders as "Syncing". It was the one chain word in this product
        // sitting on a process counter, and the fields beside it were honestly
        // `None`. It is a chain question, so it is answered from chain health.
        let health = state.workspace.list_node_health()?;
        let counted = |wanted: crate::observe::HealthState| {
            health.iter().filter(|entry| entry.state == wanted).count()
        };
        Ok(PublicStatusEnvelope {
            status: PublicStatus {
                total_nodes: nodes.len(),
                running_nodes: count_status(&nodes, NodeStatus::Running),
                syncing_nodes: counted(crate::observe::HealthState::Syncing),
                error_nodes: count_status(&nodes, NodeStatus::Error),
                total_blocks: None,
                total_peers: None,
                nodes_needing_attention: health
                    .iter()
                    .filter(|entry| entry.state.needs_attention())
                    .count(),
                timestamp: unix_millis()?,
            },
        })
    })();
    match payload {
        Ok(payload) => Json(payload).into_response(),
        Err(error) => {
            error!("NeoNexus public status failed: {error:#}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal server error" })),
            )
                .into_response()
        }
    }
}

fn count_status(nodes: &[NodeConfig], status: NodeStatus) -> usize {
    nodes.iter().filter(|node| node.status == status).count()
}

fn unix_millis() -> anyhow::Result<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| anyhow::anyhow!("system clock is before Unix epoch: {error}"))?;
    Ok(u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
}
