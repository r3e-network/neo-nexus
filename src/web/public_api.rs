//! Minimal read-only federation API.
//!
//! Only aggregate node state is public. Node identities, runtime versions,
//! host/process metrics and per-node health stay behind the operator session.

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
    timestamp: u64,
}

pub async fn status(State(state): State<WebState>) -> Response {
    let payload = (|| -> anyhow::Result<PublicStatusEnvelope> {
        let nodes = state.repository.list_nodes()?;
        Ok(PublicStatusEnvelope {
            status: PublicStatus {
                total_nodes: nodes.len(),
                running_nodes: count_status(&nodes, NodeStatus::Running),
                syncing_nodes: count_status(&nodes, NodeStatus::Starting),
                error_nodes: count_status(&nodes, NodeStatus::Error),
                total_blocks: None,
                total_peers: None,
                timestamp: unix_millis()?,
            },
        })
    })();
    match payload {
        Ok(payload) => Json(payload).into_response(),
        Err(error) => {
            eprintln!("NeoNexus public status failed: {error:#}");
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
