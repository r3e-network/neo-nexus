use super::*;
use crate::{
    agents::{AgentKind, AgentProfile, AgentRecord, AgentStatus},
    assistants::AssistantDraft,
    repository::Repository,
    types::{Network, NewNode, NodeConfig, NodeStatus, NodeType},
    web::auth::AuthStore,
};

fn fixture(operate: bool) -> (tempfile::TempDir, WebState, NodeConfig, String, String) {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("mcp.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "authorized node".into(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: "missing-node".into(),
            args: vec!["--password=private-argument".into()],
            runtime_version: "1.0".into(),
            storage_engine: NodeType::NeoRs.default_storage_engine(),
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();
    repository
        .put_agent(&AgentRecord {
            profile: AgentProfile {
                id: "hermes".into(),
                name: "Hermes".into(),
                kind: AgentKind::Hermes,
                node_id: None,
                version: "test".into(),
                binary_path: "hermes".into(),
                working_dir: directory.path().into(),
                args: vec![],
                config_path: None,
                health_url: None,
                auto_restart: false,
                binary_sha256: String::new(),
                config_sha256: None,
            },
            status: AgentStatus::Stopped,
            pid: None,
            process_started_at: None,
            desired_running: false,
            restart_attempts: 0,
            restart_after: None,
            healthy: None,
            last_health_at: 0,
        })
        .unwrap();
    let (profile, token) = repository
        .connect_assistant(AssistantDraft {
            id: String::new(),
            name: "test assistant".into(),
            agent_id: "hermes".into(),
            node_ids: vec![node.id.clone()],
            all_nodes: false,
            can_operate: operate,
        })
        .unwrap();
    let state = WebState::new(
        repository,
        directory.path().into(),
        AuthStore::from_token("operator"),
    );
    (directory, state, node, profile.id, token)
}

fn tool(state: &WebState, token: &str, name: &str, arguments: Value) -> Value {
    dispatch(
        state,
        token,
        json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
        "params":{"name":name,"arguments":arguments}}),
    )
    .unwrap()
}

fn tool_data(response: &Value) -> Value {
    serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}

#[tokio::test]
async fn mcp_batches_return_only_request_responses_and_notifications_have_no_effect() {
    let (_directory, state, node, _id, token) = fixture(true);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    let batch = json!([
        {"jsonrpc":"2.0","id":"ping","method":"ping"},
        {"jsonrpc":"2.0","method":"tools/call","params":{"name":"node_stop","arguments":{"node_id":node.id}}},
        {"jsonrpc":"2.0","id":2,"method":"tools/list"}
    ]);
    let response = post(
        State(state.clone()),
        headers.clone(),
        Bytes::from(batch.to_string()),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 65536)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(payload.as_array().unwrap().len(), 2);
    assert_eq!(payload[0]["id"], "ping");
    assert_eq!(payload[1]["id"], 2);
    assert!(!state
        .repository
        .list_recent_events(20)
        .unwrap()
        .iter()
        .any(|event| event.kind == crate::events::EventKind::AssistantToolCalled));
    let response = post(
        State(state),
        headers,
        Bytes::from("[{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}]"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    assert!(axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap()
        .is_empty());
}

#[test]
fn mcp_validates_ids_params_and_exact_tool_schemas() {
    let (_directory, state, node, _id, token) = fixture(false);
    for id in [Value::Null, json!(true), json!(1.5)] {
        let response = dispatch(
            &state,
            &token,
            json!({"jsonrpc":"2.0","id":id,"method":"ping"}),
        )
        .unwrap();
        assert_eq!(response["error"]["code"], -32600);
    }
    let response = dispatch(
        &state,
        &token,
        json!({"jsonrpc":"2.0","id":1,"method":"ping","params":[]}),
    )
    .unwrap();
    assert_eq!(response["error"]["code"], -32602);
    assert_eq!(
        tool(&state, &token, "nodes_list", json!({"node_id":node.id}))["error"]["code"],
        -32602
    );
    assert_eq!(
        tool(
            &state,
            &token,
            "node_logs",
            json!({"node_id":node.id,"limit":201})
        )["error"]["code"],
        -32602
    );
}

#[test]
fn mcp_read_scope_hides_arguments_and_health_credentials_and_refuses_operations() {
    let (_directory, state, node, _id, token) = fixture(false);
    state
        .repository
        .record_rpc_health(
            &node,
            &crate::rpc_health::RpcHealthReport {
                endpoint: "http://alice:rpc-password@127.0.0.1:20332/?token=rpc-token".into(),
                status: crate::rpc_health::RpcHealthStatus::Healthy,
                version: Some("node password=version-secret".into()),
                block_count: Some(42),
                syncing: None,
                methods: vec![],
            },
        )
        .unwrap();
    let response = tool(&state, &token, "node_status", json!({"node_id":node.id}));
    let data = tool_data(&response);
    assert_eq!(data["rpc_health"]["block_count"], 42);
    let text = response.to_string();
    for secret in [
        "rpc-password",
        "rpc-token",
        "version-secret",
        "private-argument",
        "binary_path",
    ] {
        assert!(!text.contains(secret), "exposed {secret}");
    }
    assert!(data["rpc_health"].get("endpoint").is_none());
    assert_eq!(
        tool(&state, &token, "node_stop", json!({"node_id":node.id}))["result"]["isError"],
        true
    );
    assert_eq!(
        tool(
            &state,
            &token,
            "node_status",
            json!({"node_id":"out-of-scope"})
        )["result"]["isError"],
        true
    );
    assert_eq!(
        state.repository.list_nodes().unwrap()[0].status,
        NodeStatus::Stopped
    );
}

#[test]
fn mcp_revocation_blocks_a_lifecycle_operation_waiting_for_the_supervisor() {
    let (_directory, state, node, id, token) = fixture(true);
    state
        .repository
        .update_node_status(&node.id, NodeStatus::Crashed, None)
        .unwrap();
    let engine = state.engine_state();
    let guard = engine.supervisor.lock().unwrap();
    let worker_state = state.clone();
    let node_id = node.id.clone();
    let worker = std::thread::spawn(move || {
        tool(
            &worker_state,
            &token,
            "node_stop",
            json!({"node_id":node_id}),
        )
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if state
            .repository
            .list_recent_events(20)
            .unwrap()
            .iter()
            .any(|event| event.message.contains("node_stop requested"))
        {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "operation never entered the queue"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    state.repository.revoke_assistant(&id).unwrap();
    drop(guard);
    let response = worker.join().unwrap();
    assert_eq!(response["result"]["isError"], true);
    assert!(response.to_string().contains("revoked"));
    assert_eq!(
        state.repository.list_nodes().unwrap()[0].status,
        NodeStatus::Crashed
    );
}

#[test]
fn mcp_log_redaction_keeps_multiline_secret_context_before_tail_selection() {
    let (_directory, state, node, _id, token) = fixture(false);
    let path = crate::supervisor::log_path_for(state.workspace_child_dir("logs"), &node);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, "healthy\nAuthorization:\nBearer\nsecret-bearer-value\npassword:\nsecret-password-value\n-----BEGIN EC PRIVATE KEY-----\nsecret-pem-material\n-----END EC PRIVATE KEY-----\n").unwrap();
    let response = tool(
        &state,
        &token,
        "node_logs",
        json!({"node_id":node.id,"limit":1}),
    );
    assert_eq!(response["result"]["isError"], false);
    let text = response.to_string();
    for secret in [
        "secret-bearer-value",
        "secret-password-value",
        "secret-pem-material",
    ] {
        assert!(!text.contains(secret), "exposed {secret}");
    }
    assert_eq!(tool_data(&response)["untrusted_data"], true);
}

#[test]
fn mcp_guarded_launch_rejects_plugin_changes_before_config_or_process_effects() {
    let (_directory, state, original, _id, _token) = fixture(true);
    let rpc = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let p2p = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let node = state
        .repository
        .update_node(
            &original.id,
            NewNode {
                name: original.name,
                node_type: original.node_type,
                network: original.network,
                binary_path: std::env::current_exe().unwrap(),
                args: vec![],
                runtime_version: "1.0".into(),
                storage_engine: original.storage_engine,
                rpc_port: rpc.local_addr().unwrap().port(),
                p2p_port: p2p.local_addr().unwrap().port(),
                ws_port: None,
            },
        )
        .unwrap();
    drop((rpc, p2p));
    let plugin = crate::core::runtime::PluginId::RpcServer;
    let enabled = state
        .repository
        .list_plugin_states(&node.id)
        .unwrap()
        .iter()
        .any(|state| state.plugin_id == plugin && state.enabled);
    let result = crate::supervision::launch_node_guarded(
        &state.engine_state(),
        &node,
        crate::node_lifecycle::LaunchAction::Start,
        || {
            state
                .repository
                .set_plugin_enabled(&node.id, plugin, !enabled)
        },
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("plugin configuration changed"));
    assert_eq!(
        state.repository.list_nodes().unwrap()[0].status,
        NodeStatus::Stopped
    );
    assert!(!crate::config::ConfigExporter::managed_target_path(
        state.workspace_child_dir("nodes").join(&node.id),
        &node
    )
    .exists());
}
