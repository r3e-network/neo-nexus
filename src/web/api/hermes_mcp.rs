//! Node-Scoped MCP (Model Context Protocol) & Hermes Guest Agent API.
//!
//! Provides JSON-RPC 2.0 tool execution and heartbeat telemetry for Nous Research
//! Hermes Agent, strictly scoped to a single node instance.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    wallet::TokenPermission,
    web::{api_tokens::AuthIdentity, WebState},
};

#[derive(Deserialize)]
pub struct McpRequest {
    #[serde(default)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Deserialize)]
pub struct HeartbeatPayload {
    #[serde(default)]
    pub agent_version: Option<String>,
}

/// Verify that the authenticated identity is authorized for this specific node.
fn check_node_authorization(identity: Option<&AuthIdentity>, node_id: &str) -> bool {
    match identity {
        Some(AuthIdentity::Session) => true,
        Some(AuthIdentity::Token(token)) => {
            token.has_permission(&TokenPermission::AdminAll)
                || token.has_permission(&TokenPermission::HermesAgent(node_id.to_string()))
        }
        None => false,
    }
}

/// POST /api/nodes/{id}/mcp
pub async fn mcp_endpoint(
    State(state): State<WebState>,
    Path(id): Path<String>,
    identity: Option<Extension<AuthIdentity>>,
    Json(req): Json<McpRequest>,
) -> Response {
    let auth = identity.as_ref().map(|ext| &ext.0);
    if !check_node_authorization(auth, &id) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "jsonrpc": "2.0",
                "id": req.id,
                "error": { "code": -32000, "message": "Access denied: token is not scoped to this node instance" }
            })),
        )
            .into_response();
    }

    let node = match state
        .workspace
        .list_nodes()
        .ok()
        .and_then(|nodes| nodes.into_iter().find(|n| n.id == id))
    {
        Some(n) => n,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "error": { "code": -32004, "message": format!("Node {id} not found") }
                })),
            )
                .into_response();
        }
    };

    // The guest agent is a per-instance feature the operator switches on, so an
    // instance that has never had one — or has had it turned off — exposes no
    // tool surface at all. Holding a credential is not the same as the operator
    // having enabled the agent, and absence must read as off rather than as
    // "not configured, so allow".
    let association = state.workspace.load_hermes_agent(&node.id).ok().flatten();
    if !association.as_ref().is_some_and(|a| a.enabled) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "jsonrpc": "2.0",
                "id": req.id,
                "error": {
                    "code": -32002,
                    "message": format!(
                        "The guest agent is not enabled for instance {}. Enable it on the instance before its copilot can act.",
                        node.name
                    )
                }
            })),
        )
            .into_response();
    }

    // Everything a credential confined to this instance must not reach, even
    // through a tool call it is otherwise entitled to make.
    let confined = auth.is_some_and(|identity| identity.confined_to_node().is_some());

    match req.method.as_str() {
        "tools/list" => super::hermes_tools::list(&req, confined),
        "tools/call" => {
            super::hermes_tools::call(&state, &node, &req, confined, association.as_ref()).await
        }
        other => Json(json!({
            "jsonrpc": "2.0",
            "id": req.id,
            "error": { "code": -32601, "message": format!("Method '{other}' not supported") }
        }))
        .into_response(),
    }
}

/// POST /api/nodes/{id}/agent/heartbeat
pub async fn agent_heartbeat(
    State(state): State<WebState>,
    Path(id): Path<String>,
    identity: Option<Extension<AuthIdentity>>,
    Json(payload): Json<HeartbeatPayload>,
) -> Response {
    let auth = identity.as_ref().map(|ext| &ext.0);
    if !check_node_authorization(auth, &id) {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }
    match state
        .commands
        .record_hermes_heartbeat(&id, payload.agent_version.as_deref())
    {
        Ok(true) => {
            let assoc = state.workspace.load_hermes_agent(&id).ok().flatten();
            Json(json!({
                "status": "ok",
                "node_id": id,
                // Reported, never assumed: an agent reads this to decide whether
                // it is allowed to act on its own, so an absent grant has to
                // come back as false.
                "autonomous_healing": assoc.is_some_and(|a| a.autonomous_healing),
            }))
            .into_response()
        }
        // A heartbeat is a report from an agent the operator enrolled. It is not
        // itself an enrolment: accepting one for an instance with no association
        // used to create that association with self-healing already switched on.
        Ok(false) => (
            StatusCode::FORBIDDEN,
            format!("the guest agent is not enabled for instance {id}"),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// GET /api/nodes/{id}/agent
pub async fn agent_status(
    State(state): State<WebState>,
    Path(id): Path<String>,
    identity: Option<Extension<AuthIdentity>>,
) -> Response {
    let auth = identity.as_ref().map(|ext| &ext.0);
    if !check_node_authorization(auth, &id) {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    // An instance with no association is not enrolled. Standing in a default
    // `HermesAgentAssociation` here answered `enabled: true` and
    // `autonomous_healing: true` for exactly that case — an agent reads this to
    // decide what it may do, so the unenrolled state has to be reported as
    // itself rather than as a permissive default.
    let Some(assoc) = state.workspace.load_hermes_agent(&id).ok().flatten() else {
        return Json(json!({
            "node_id": id,
            "enrolled": false,
            "enabled": false,
            "autonomous_healing": false,
            "agent_version": Value::Null,
            "last_heartbeat_unix": Value::Null,
            "is_alive": false,
        }))
        .into_response();
    };
    let is_alive = assoc.is_alive(now);

    Json(json!({
        "node_id": assoc.node_id,
        "enrolled": true,
        "enabled": assoc.enabled,
        "autonomous_healing": assoc.autonomous_healing,
        "agent_version": assoc.agent_version,
        "last_heartbeat_unix": assoc.last_heartbeat_unix,
        "is_alive": is_alive,
    }))
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::ApiToken;

    #[test]
    fn session_identity_authorizes_any_node() {
        let session = AuthIdentity::Session;
        assert!(check_node_authorization(Some(&session), "node-1"));
        assert!(check_node_authorization(Some(&session), "node-2"));
    }

    #[test]
    fn unauthenticated_is_rejected() {
        assert!(!check_node_authorization(None, "node-1"));
    }

    #[test]
    fn admin_all_token_authorizes_any_node() {
        let (token, _) = ApiToken::generate("admin-token", vec![TokenPermission::AdminAll]);
        let auth = AuthIdentity::Token(Box::new(token));
        assert!(check_node_authorization(Some(&auth), "node-1"));
        assert!(check_node_authorization(Some(&auth), "node-2"));
    }

    #[test]
    fn hermes_agent_token_is_strictly_scoped_to_its_node() {
        let (token, _) = ApiToken::generate(
            "hermes-node-1",
            vec![TokenPermission::HermesAgent("node-1".to_string())],
        );
        let auth = AuthIdentity::Token(Box::new(token));
        assert!(check_node_authorization(Some(&auth), "node-1"));
        assert!(!check_node_authorization(Some(&auth), "node-2"));
    }

    #[test]
    fn read_fleet_token_cannot_access_mcp() {
        let (token, _) = ApiToken::generate("read-only", vec![TokenPermission::ReadFleet]);
        let auth = AuthIdentity::Token(Box::new(token));
        assert!(!check_node_authorization(Some(&auth), "node-1"));
    }

    #[test]
    fn hermes_config_snippet_formats_mcp_yaml() {
        let snippet = crate::agents::generate_hermes_config_snippet(
            "node-abc-123",
            "alpha",
            "http://127.0.0.1:8080/api/nodes/node-abc-123/mcp",
        );
        assert!(snippet.contains("neonexus_node_abc_123:"));
        assert!(snippet.contains("get_node_config"));
        assert!(snippet.contains("smoke_test_node"));
        assert!(snippet.contains("get_node_iac"));
        // The snippet documents what this credential can do, and the endpoint
        // refuses a whole-workspace export to it. Listing the tool as granted
        // would send an operator looking for a capability that is not there.
        assert!(snippet.contains("Deliberately NOT granted: take_snapshot"));
    }
}
