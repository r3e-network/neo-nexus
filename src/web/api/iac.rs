//! REST API endpoints for single-node IaC export and fleet-wide cluster manifests.

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::web::{
    pages::nodes::{
        generate_fleet_cloudformation, generate_fleet_compose, generate_fleet_k8s,
        generate_fleet_terraform, generate_node_iac, IacFormat,
    },
    WebState,
};

#[derive(Deserialize)]
pub struct IacQuery {
    #[serde(default)]
    pub format: Option<String>,
}

/// GET /api/nodes/{id}/iac?format=k8s|docker|json|cli|cloudformation|terraform
pub async fn node_iac(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Query(query): Query<IacQuery>,
) -> Response {
    let nodes = match state.workspace.list_nodes() {
        Ok(nodes) => nodes,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": err.to_string() })),
            )
                .into_response()
        }
    };
    let node = match nodes.into_iter().find(|n| n.id == id) {
        Some(node) => node,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Node {id} not found") })),
            )
                .into_response()
        }
    };

    let role = state.workspace.load_node_role(&node.id).ok().flatten();
    let signer = state
        .workspace
        .load_node_signer_key(&node.id)
        .ok()
        .flatten();
    let assoc = state.workspace.load_hermes_agent(&node.id).ok().flatten();

    let format_str = query.format.as_deref().unwrap_or("json");
    let format = format_str.parse::<IacFormat>().unwrap_or(IacFormat::Json);

    let (content, mime, ext) =
        generate_node_iac(&node, role, signer.as_ref(), assoc.as_ref(), format);

    let safe_name = node.name.to_lowercase().replace(' ', "-");
    let filename = format!("node-{safe_name}.{ext}");

    let mut response = content.into_response();
    let headers = response.headers_mut();
    if let Ok(val) = HeaderValue::from_str(mime) {
        headers.insert(header::CONTENT_TYPE, val);
    }
    if let Ok(val) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        headers.insert(header::CONTENT_DISPOSITION, val);
    }
    response
}

#[derive(Deserialize)]
pub struct FleetIacQuery {
    #[serde(default)]
    pub format: Option<String>,
}

/// GET /api/fleet/iac?format=compose|k8s|cloudformation|terraform
pub async fn fleet_iac(
    State(state): State<WebState>,
    Query(query): Query<FleetIacQuery>,
) -> Response {
    let nodes = match state.workspace.list_nodes() {
        Ok(nodes) => nodes,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": err.to_string() })),
            )
                .into_response()
        }
    };

    let format_str = query.format.as_deref().unwrap_or("compose");
    match format_str.to_ascii_lowercase().as_str() {
        "k8s" | "kubernetes" | "yaml" => {
            let manifest = generate_fleet_k8s(&nodes);
            let mut response = manifest.into_response();
            let headers = response.headers_mut();
            if let Ok(val) = HeaderValue::from_str("application/x-yaml") {
                headers.insert(header::CONTENT_TYPE, val);
            }
            if let Ok(val) = HeaderValue::from_str("attachment; filename=\"k8s-fleet.yaml\"") {
                headers.insert(header::CONTENT_DISPOSITION, val);
            }
            response
        }
        "cfn" | "cloudformation" | "aws" => {
            let manifest = generate_fleet_cloudformation(&nodes);
            let mut response = manifest.into_response();
            let headers = response.headers_mut();
            if let Ok(val) = HeaderValue::from_str("application/x-yaml") {
                headers.insert(header::CONTENT_TYPE, val);
            }
            if let Ok(val) =
                HeaderValue::from_str("attachment; filename=\"fleet-cloudformation.yaml\"")
            {
                headers.insert(header::CONTENT_DISPOSITION, val);
            }
            response
        }
        "tf" | "terraform" | "hcl" => {
            let manifest = generate_fleet_terraform(&nodes);
            let mut response = manifest.into_response();
            let headers = response.headers_mut();
            if let Ok(val) = HeaderValue::from_str("application/x-tf") {
                headers.insert(header::CONTENT_TYPE, val);
            }
            if let Ok(val) = HeaderValue::from_str("attachment; filename=\"fleet-main.tf\"") {
                headers.insert(header::CONTENT_DISPOSITION, val);
            }
            response
        }
        _ => {
            let compose = generate_fleet_compose(&nodes);
            let mut response = compose.into_response();
            let headers = response.headers_mut();
            if let Ok(val) = HeaderValue::from_str("application/x-yaml") {
                headers.insert(header::CONTENT_TYPE, val);
            }
            if let Ok(val) = HeaderValue::from_str("attachment; filename=\"docker-compose.yml\"") {
                headers.insert(header::CONTENT_DISPOSITION, val);
            }
            response
        }
    }
}
