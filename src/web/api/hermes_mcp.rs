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
    core::operations::{EventKind, EventSeverity, NewRuntimeEvent},
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
        "tools/list" => {
            let tools = json!([
                {
                    "name": "get_node_status",
                    "description": "Inspect real-time health, block height, peer count, and sync status for this node",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "get_node_config",
                    "description": "Inspect declarative configuration, role, ports, network, storage engine, and signer lease for this node",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "get_node_logs",
                    "description": "Fetch recent log observations and tail output from this node",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "lines": { "type": "integer", "default": 50 }
                        }
                    }
                },
                {
                    "name": "restart_node",
                    "description": "Perform an autonomous safe restart of the node instance (requires autonomous healing)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "reason": { "type": "string", "default": "Hermes autonomous recovery" }
                        }
                    }
                },
                {
                    "name": "stop_node",
                    "description": "Gracefully stop this node instance (quiesce process and release ports)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "reason": { "type": "string", "default": "Hermes autonomous shutdown" }
                        }
                    }
                },
                {
                    "name": "start_node",
                    "description": "Start this node instance with supervised monitoring",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "smoke_test_node",
                    "description": "Execute SRE binary smoke sweep and health diagnostic checks against this node instance",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "get_node_iac",
                    "description": "Export cloud launch template and IaC manifests (AWS CloudFormation, Terraform HCL, Kubernetes Pod YAML, Docker run command, or JSON spec)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "format": {
                                "type": "string",
                                "enum": ["cloudformation", "terraform", "k8s", "docker", "json", "cli"],
                                "default": "k8s"
                            }
                        }
                    }
                }
            ]);
            let mut tools = match tools {
                Value::Array(tools) => tools,
                other => vec![other],
            };
            // `take_snapshot` writes a whole-workspace export — every instance's
            // configuration, roles, wallet profiles and signer leases in one
            // file. That is an operator action, not something one instance's
            // copilot may trigger, so a confined credential is not offered it
            // and is refused if it asks anyway.
            if !confined {
                tools.push(json!({
                    "name": "take_snapshot",
                    "description": "Create a point-in-time safety backup of the whole workspace before upgrades or dangerous operations",
                    "inputSchema": { "type": "object", "properties": {} }
                }));
            }
            Json(json!({
                "jsonrpc": "2.0",
                "id": req.id,
                "result": { "tools": Value::Array(tools) }
            }))
            .into_response()
        }
        "tools/call" => {
            let tool_name = req.params.get("name").and_then(Value::as_str).unwrap_or("");
            match tool_name {
                "get_node_status" => {
                    let health = if node.rpc_port > 0 {
                        let report = crate::rpc_health::probe_node_rpc(
                            &node,
                            std::time::Duration::from_millis(500),
                        );
                        format!("{} ({})", report.status.label(), report.message())
                    } else {
                        "p2p_only_hardened (RPC disabled)".to_string()
                    };
                    let assoc = state.workspace.load_hermes_agent(&node.id).ok().flatten();
                    let healing_status = assoc.is_none_or(|a| a.autonomous_healing);
                    let signer_binding = state
                        .workspace
                        .load_node_signer_key(&node.id)
                        .ok()
                        .flatten();
                    let signer_desc = signer_binding.map_or_else(
                        || "unbound".to_string(),
                        |k| format!("{}/{}", k.backend_id, k.key_id),
                    );
                    let status_text = format!(
                        "Node: {}\nStatus: {}\nClient: {}\nNetwork: {}\nP2P Port: {}\nRPC Port: {}\nRPC Health: {}\nPID: {:?}\nSigner Lease: {}\nAutonomous Self-Healing: {}",
                        node.name,
                        node.status.label(),
                        node.node_type,
                        node.network,
                        node.p2p_port,
                        if node.rpc_port == 0 { "disabled".to_string() } else { node.rpc_port.to_string() },
                        health,
                        node.pid,
                        signer_desc,
                        healing_status,
                    );
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "content": [{ "type": "text", "text": status_text }]
                        }
                    }))
                    .into_response()
                }
                "get_node_config" => {
                    let role = state.workspace.load_node_role(&node.id).ok().flatten();
                    let signer = state
                        .workspace
                        .load_node_signer_key(&node.id)
                        .ok()
                        .flatten();
                    let assoc = state.workspace.load_hermes_agent(&node.id).ok().flatten();
                    let config_json = serde_json::json!({
                        "id": node.id,
                        "name": node.name,
                        "client": node.node_type.to_string(),
                        "network": node.network.to_string(),
                        "role": role.map(|r| r.slug().to_string()).unwrap_or_else(|| "observer".to_string()),
                        "role_label": role.map(|r| r.label().to_string()).unwrap_or_else(|| "Node".to_string()),
                        "runtime_version": node.runtime_version,
                        "storage_engine": node.storage_engine.to_string(),
                        "ports": {
                            "p2p": node.p2p_port,
                            "rpc": if node.rpc_port == 0 { None } else { Some(node.rpc_port) },
                            "ws": node.ws_port,
                        },
                        "signer_lease": signer.map(|k| serde_json::json!({
                            "backend_id": k.backend_id,
                            "key_id": k.key_id,
                        })),
                        "hermes_copilot": assoc.map(|a| serde_json::json!({
                            "enabled": a.enabled,
                            "autonomous_healing": a.autonomous_healing,
                            "agent_version": a.agent_version,
                        })),
                        "binary_path": node.binary_path.display().to_string(),
                        "args": node.args,
                    });
                    let text = serde_json::to_string_pretty(&config_json)
                        .unwrap_or_else(|_| "{}".to_string());
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "content": [{ "type": "text", "text": text }]
                        }
                    }))
                    .into_response()
                }
                "get_node_logs" => {
                    let log_dir = state.workspace_child_dir("logs");
                    let log_path = crate::core::runtime::log_path_for(log_dir, &node);
                    let text = match crate::logs::LogReader::snapshot(&log_path, 64 * 1024) {
                        Ok(snapshot) => {
                            let lines: Vec<String> = snapshot
                                .lines
                                .iter()
                                .rev()
                                .take(50)
                                .rev()
                                .cloned()
                                .collect();
                            if lines.is_empty() {
                                "No recent log output recorded for this node.".to_string()
                            } else {
                                lines.join("\n")
                            }
                        }
                        Err(e) => format!("Could not read logs for {}: {e}", node.id),
                    };
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "content": [{ "type": "text", "text": text }]
                        }
                    }))
                    .into_response()
                }
                "restart_node" => {
                    // Letting a machine restart a validator is a grant the
                    // operator makes, so it has to be present to count. The
                    // previous reading treated a missing association as consent.
                    let can_heal = association
                        .as_ref()
                        .is_some_and(|assoc| assoc.autonomous_healing);
                    if !can_heal {
                        return (
                            StatusCode::FORBIDDEN,
                            Json(json!({
                                "jsonrpc": "2.0",
                                "id": req.id,
                                "error": { "code": -32001, "message": "Autonomous self-healing is disabled for this node instance" }
                            })),
                        )
                            .into_response();
                    }
                    // Industry-Standard Circuit Breaker: prevent crash loop restart storm
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let recent_restarts = state
                        .workspace
                        .list_events(crate::core::operations::RuntimeEventFilter::new(
                            None, &node.id, 20,
                        ))
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|e| {
                            e.kind == EventKind::NodeRestarted
                                && now.saturating_sub(e.occurred_at_unix) < 3600
                        })
                        .count();

                    if recent_restarts >= 5 {
                        let _ = state.commands.record_event(NewRuntimeEvent {
                            node_id: Some(node.id.clone()),
                            node_name: Some(node.name.clone()),
                            kind: EventKind::NodeStartFailed,
                            severity: EventSeverity::Critical,
                            message: format!("Hermes autonomous restart tripped circuit breaker: exceeded budget ({recent_restarts}/5 restarts in last hour)"),
                        });
                        return Json(json!({
                            "jsonrpc": "2.0",
                            "id": req.id,
                            "error": {
                                "code": -32000,
                                "message": format!("Circuit breaker active: restart rate limit exceeded ({recent_restarts}/5 restarts in the last hour). Manual operator investigation required.")
                            }
                        }))
                        .into_response();
                    }

                    let reason = req
                        .params
                        .get("arguments")
                        .and_then(|a| a.get("reason"))
                        .and_then(Value::as_str)
                        .unwrap_or("Hermes autonomous self-healing restart");
                    let outcome = crate::supervision::launch_node(
                        &state.engine_state(),
                        &node,
                        crate::core::lifecycle::LaunchAction::Restart,
                    );
                    let (is_ok, msg) = match outcome {
                        Ok(m) => (true, m),
                        Err(e) => (false, e.to_string()),
                    };
                    let _ = state.commands.record_event(NewRuntimeEvent {
                        node_id: Some(node.id.clone()),
                        node_name: Some(node.name.clone()),
                        kind: if is_ok {
                            EventKind::NodeRestarted
                        } else {
                            EventKind::NodeStartFailed
                        },
                        severity: if is_ok {
                            EventSeverity::Warning
                        } else {
                            EventSeverity::Critical
                        },
                        message: format!("Hermes Agent executed restart: {reason} — {msg}"),
                    });
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "content": [{ "type": "text", "text": format!("Autonomous restart outcome: {msg} ({reason})") }]
                        }
                    }))
                    .into_response()
                }
                "stop_node" => {
                    let reason = req
                        .params
                        .get("arguments")
                        .and_then(|a| a.get("reason"))
                        .and_then(Value::as_str)
                        .unwrap_or("Hermes autonomous stop");
                    let outcome = crate::supervision::stop_node(&state.engine_state(), &node);
                    let (_is_ok, msg) = match outcome {
                        Ok(m) => (true, m),
                        Err(e) => (false, e.to_string()),
                    };
                    let _ = state.commands.record_event(NewRuntimeEvent {
                        node_id: Some(node.id.clone()),
                        node_name: Some(node.name.clone()),
                        kind: EventKind::NodeStopped,
                        severity: EventSeverity::Info,
                        message: format!("Hermes Agent executed stop: {reason} — {msg}"),
                    });
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "content": [{ "type": "text", "text": format!("Autonomous stop outcome: {msg} ({reason})") }]
                        }
                    }))
                    .into_response()
                }
                "start_node" => {
                    let outcome = crate::supervision::launch_node(
                        &state.engine_state(),
                        &node,
                        crate::core::lifecycle::LaunchAction::Start,
                    );
                    let (is_ok, msg) = match outcome {
                        Ok(m) => (true, m),
                        Err(e) => (false, e.to_string()),
                    };
                    let _ = state.commands.record_event(NewRuntimeEvent {
                        node_id: Some(node.id.clone()),
                        node_name: Some(node.name.clone()),
                        kind: if is_ok {
                            EventKind::NodeStarted
                        } else {
                            EventKind::NodeStartFailed
                        },
                        severity: if is_ok {
                            EventSeverity::Info
                        } else {
                            EventSeverity::Critical
                        },
                        message: format!("Hermes Agent executed start — {msg}"),
                    });
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "content": [{ "type": "text", "text": format!("Autonomous start outcome: {msg}") }]
                        }
                    }))
                    .into_response()
                }
                "take_snapshot" => {
                    if confined {
                        return (
                            StatusCode::FORBIDDEN,
                            Json(json!({
                                "jsonrpc": "2.0",
                                "id": req.id,
                                "error": {
                                    "code": -32003,
                                    "message": "take_snapshot exports the whole workspace, including every other instance. A credential scoped to one instance cannot request it."
                                }
                            })),
                        )
                            .into_response();
                    }
                    let backup_dir = state.workspace_child_dir("export").join("backup");
                    let outcome = state
                        .workspace
                        .export_backup(&backup_dir, env!("CARGO_PKG_VERSION"));
                    let (is_ok, msg) = match outcome {
                        Ok(export) => (
                            true,
                            format!(
                                "Safety snapshot created successfully: {} ({} bytes, {} nodes)",
                                export.path.display(),
                                export.bytes_written,
                                export.node_count
                            ),
                        ),
                        Err(e) => (false, format!("Failed to create safety snapshot: {e}")),
                    };
                    if is_ok {
                        let _ = state.commands.record_event(NewRuntimeEvent {
                            node_id: Some(node.id.clone()),
                            node_name: Some(node.name.clone()),
                            kind: EventKind::BackupExported,
                            severity: EventSeverity::Info,
                            message: format!("Hermes Agent executed safety snapshot: {msg}"),
                        });
                    }
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "content": [{ "type": "text", "text": msg }]
                        }
                    }))
                    .into_response()
                }
                "smoke_test_node" => {
                    let node_config = node.clone();
                    let report = tokio::task::spawn_blocking(move || {
                        crate::runtime_smoke::smoke_node_binary(
                            &node_config,
                            std::time::Duration::from_secs(3),
                        )
                    })
                    .await
                    .map_err(|e| anyhow::anyhow!("smoke task failed: {e}"));

                    let (pass, msg) = match report {
                        Ok(r) => (r.status.is_success(), r.message),
                        Err(e) => (false, e.to_string()),
                    };

                    let text = format!(
                        "SRE Smoke Test Result: {}\nNode: {}\nClient: {}\nDetails: {}",
                        if pass { "PASSED" } else { "FAILED" },
                        node.name,
                        node.node_type,
                        msg
                    );
                    let _ = state.commands.record_event(NewRuntimeEvent {
                        node_id: Some(node.id.clone()),
                        node_name: Some(node.name.clone()),
                        kind: EventKind::RuntimeSmokeTested,
                        severity: if pass {
                            EventSeverity::Info
                        } else {
                            EventSeverity::Warning
                        },
                        message: format!(
                            "Hermes autonomous smoke sweep: {} — {}",
                            if pass { "PASSED" } else { "FAILED" },
                            msg
                        ),
                    });
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "content": [{ "type": "text", "text": text }]
                        }
                    }))
                    .into_response()
                }
                "get_node_iac" => {
                    let role = state.workspace.load_node_role(&node.id).ok().flatten();
                    let signer = state
                        .workspace
                        .load_node_signer_key(&node.id)
                        .ok()
                        .flatten();
                    let assoc = state.workspace.load_hermes_agent(&node.id).ok().flatten();
                    let format_arg = req
                        .params
                        .get("arguments")
                        .and_then(|a| a.get("format"))
                        .and_then(Value::as_str)
                        .unwrap_or("k8s");
                    let format = format_arg
                        .parse::<crate::web::pages::nodes::IacFormat>()
                        .unwrap_or(crate::web::pages::nodes::IacFormat::K8s);
                    let (spec, _, _) = crate::web::pages::nodes::generate_node_iac(
                        &node,
                        role,
                        signer.as_ref(),
                        assoc.as_ref(),
                        format,
                    );
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "content": [{ "type": "text", "text": spec }]
                        }
                    }))
                    .into_response()
                }
                other => Json(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "error": { "code": -32601, "message": format!("Unknown tool '{other}'") }
                }))
                .into_response(),
            }
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
