use super::*;
use neo_nexus::{
    agents::{self, AgentKind, AgentProfile},
    assistants::{self, AssistantDraft},
    events::{EventKind, EventSeverity, NewRuntimeEvent},
};
use serde_json::{json, Value};

fn hermes(server: &Server) -> PathBuf {
    let root = server._home.path().join("hermes");
    std::fs::create_dir_all(root.join("hermes_cli")).unwrap();
    std::fs::write(
        root.join("hermes_cli/main.py"),
        "# fixture, never executed\n",
    )
    .unwrap();
    let python = root.join(if cfg!(windows) {
        "python.exe"
    } else {
        "python"
    });
    std::fs::write(&python, "fixture, never executed").unwrap();
    let config = root.join("config.yaml");
    std::fs::write(&config, "model: existing-model\nplatforms:\n  telegram:\n    enabled: true\nmcp_servers:\n  existing:\n    command: existing-server\n").unwrap();
    std::fs::write(
        root.join(".env"),
        "TELEGRAM_BOT_TOKEN=existing-channel-token\n",
    )
    .unwrap();
    agents::save(
        &server.state.engine_state(),
        AgentProfile {
            id: "hermes".into(),
            name: "Existing Hermes".into(),
            kind: AgentKind::Hermes,
            node_id: None,
            version: "0.21.0".into(),
            binary_path: python,
            working_dir: root,
            args: vec![],
            config_path: Some(config.clone()),
            health_url: None,
            auto_restart: true,
            binary_sha256: String::new(),
            config_sha256: None,
        },
    )
    .unwrap();
    config
}

fn credential(server: &Server, node: &str, can_operate: bool) -> (String, String) {
    let (profile, token) = assistants::connect(
        &server.state.engine_state(),
        AssistantDraft {
            id: String::new(),
            name: "Node assistant".into(),
            agent_id: "hermes".into(),
            node_ids: vec![node.into()],
            all_nodes: false,
            can_operate,
        },
    )
    .unwrap();
    (profile.id, token)
}

fn request(server: &Server, token: &str, body: Value) -> ureq::Response {
    into_response(
        agent()
            .post(&format!("{}/mcp", server.base_url))
            .set("Authorization", &format!("Bearer {token}"))
            .set("Content-Type", "application/json")
            .set("Accept", "application/json, text/event-stream")
            .send_string(&body.to_string()),
    )
}

fn call(server: &Server, token: &str, name: &str, arguments: Value) -> Value {
    json_body(request(
        server,
        token,
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":name,"arguments":arguments}}),
    ))
}

fn content(response: &Value) -> Value {
    serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}

#[test]
fn mcp_has_its_own_authentication_and_implements_stateless_http() {
    let server = spawn_server();
    hermes(&server);
    let node = create_node(&server.db_path, "scope", 41002);
    let (_, token) = credential(&server, &node, false);
    let initialize = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"hermes-test","version":"1"}}});
    assert_eq!(request(&server, TOKEN, initialize.clone()).status(), 401);
    let initialized = json_body(request(&server, &token, initialize));
    assert_eq!(initialized["result"]["protocolVersion"], "2025-03-26");
    assert_eq!(initialized["result"]["serverInfo"]["name"], "NeoNexus");
    assert_eq!(
        request(
            &server,
            &token,
            json!({"jsonrpc":"2.0","method":"notifications/initialized"})
        )
        .status(),
        202
    );
    let origin = into_response(
        agent()
            .post(&format!("{}/mcp", server.base_url))
            .set("Authorization", &format!("Bearer {token}"))
            .set("Origin", "https://attacker.example")
            .send_string("{}"),
    );
    assert_eq!(origin.status(), 403);
    assert_eq!(
        into_response(agent().get(&format!("{}/mcp", server.base_url)).call()).status(),
        405
    );
    let malformed = json_body(request(&server, &token, json!([])));
    assert_eq!(malformed["error"]["code"], -32600);
}

#[test]
fn assistant_scope_readonly_redaction_and_revocation_are_enforced_by_the_server() {
    let server = spawn_server();
    hermes(&server);
    let allowed = create_node(&server.db_path, "visible-node", 41012);
    let hidden = create_node(&server.db_path, "private-node", 41022);
    let (grant, token) = credential(&server, &allowed, false);
    let tools = json_body(request(
        &server,
        &token,
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    ));
    assert!(!tools.to_string().contains("node_restart"));
    let nodes = content(&call(&server, &token, "nodes_list", json!({})));
    assert_eq!(nodes.as_array().unwrap().len(), 1);
    assert_eq!(nodes[0]["id"], allowed);
    let denied = call(&server, &token, "node_status", json!({"node_id":hidden}));
    assert_eq!(denied["result"]["isError"], true);
    assert!(!denied.to_string().contains("private-node"));
    assert_eq!(
        call(&server, &token, "node_stop", json!({"node_id":allowed}))["result"]["isError"],
        true
    );
    let node = server
        .state
        .repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == allowed)
        .unwrap();
    let path = neo_nexus::core::runtime::log_path_for(server._home.path().join("logs"), &node);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, "password=secret-log-value\nnormal status\n").unwrap();
    let logs = call(
        &server,
        &token,
        "node_logs",
        json!({"node_id":allowed,"limit":2}),
    );
    assert!(!logs.to_string().contains("secret-log-value"));
    assert!(logs.to_string().contains("normal status"));
    for (id, message) in [
        (&allowed, "visible event"),
        (&hidden, "private-event-must-not-leak"),
    ] {
        server
            .state
            .repository
            .record_event(NewRuntimeEvent {
                node_id: Some(id.clone()),
                node_name: None,
                kind: EventKind::NodeExited,
                severity: EventSeverity::Warning,
                message: message.into(),
            })
            .unwrap();
    }
    let events = call(&server, &token, "node_events", json!({"node_id":allowed}));
    assert!(events.to_string().contains("visible event"));
    assert!(!events.to_string().contains("private-event-must-not-leak"));
    assert_eq!(
        call(
            &server,
            &token,
            "node_logs",
            json!({"node_id":allowed,"limit":201})
        )["error"]["code"],
        -32602
    );
    assert_eq!(
        call(&server, &token, "shell", json!({"command":"whoami"}))["error"]["code"],
        -32602
    );
    assistants::revoke(&server.state.repository, &grant).unwrap();
    assert_eq!(
        request(
            &server,
            &token,
            json!({"jsonrpc":"2.0","id":1,"method":"ping"})
        )
        .status(),
        401
    );
    let database = rusqlite::Connection::open(&server.db_path).unwrap();
    let digest: String = database
        .query_row(
            "SELECT token_sha256 FROM assistant_grants WHERE id=?1",
            [&grant],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(digest.len(), 64);
    assert_ne!(digest, token);
}

#[test]
fn operator_tools_start_restart_and_stop_the_real_supervised_process() {
    let server = spawn_server();
    hermes(&server);
    let (binary_path, args) = long_running_command();
    let node = server
        .state
        .repository
        .create_node(NewNode {
            name: "assistant-managed".into(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path,
            args,
            runtime_version: "test".into(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 41032,
            p2p_port: 41033,
            ws_port: None,
        })
        .unwrap();
    let (_, token) = credential(&server, &node.id, true);
    let start = call(&server, &token, "node_start", json!({"node_id":node.id}));
    assert_eq!(start["result"]["isError"], false, "{start}");
    let first = server.state.repository.list_nodes().unwrap()[0]
        .pid
        .unwrap();
    assert!(process_alive(first));
    let restart = call(&server, &token, "node_restart", json!({"node_id":node.id}));
    assert_eq!(restart["result"]["isError"], false, "{restart}");
    let second = server.state.repository.list_nodes().unwrap()[0]
        .pid
        .unwrap();
    assert_ne!(first, second);
    assert!(!process_alive(first));
    assert!(process_alive(second));
    let stop = call(&server, &token, "node_stop", json!({"node_id":node.id}));
    assert_eq!(stop["result"]["isError"], false, "{stop}");
    assert!(!process_alive(second));
    assert_eq!(server.state.repository.list_nodes().unwrap()[0].pid, None);
    let events = server
        .state
        .repository
        .list_node_events(&node.id, 50)
        .unwrap();
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::AssistantToolCalled
            && event.message.contains("node_stop completed")));
}

#[test]
fn connecting_in_the_workbench_preserves_hermes_channels_and_never_renders_credentials() {
    let server = spawn_server();
    let config = hermes(&server);
    let node = create_node(&server.db_path, "connected-node", 41042);
    let http = agent();
    let cookie = signed_in(&http, &server.base_url);
    let url = format!("{}/assistants/connect", server.base_url);
    let body = format!(
        "id=connection&name=Hermes&agent_id=hermes&node_{node}=true&endpoint={}",
        html::urlencoding_lite(&format!("{}/mcp", server.base_url))
    );
    assert_eq!(
        post_form(&http, &url, &body).header("location"),
        Some("/login")
    );
    let saved = post_form_as(&http, &cookie, &url, &body);
    assert!(
        saved.header("location").unwrap().contains("configured"),
        "{:?}",
        saved.header("location")
    );
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
    assert_eq!(
        yaml["platforms"]["telegram"]["enabled"].as_bool(),
        Some(true)
    );
    assert_eq!(yaml["model"].as_str(), Some("existing-model"));
    assert_eq!(
        yaml["mcp_servers"]["existing"]["command"].as_str(),
        Some("existing-server")
    );
    let environment = std::fs::read_to_string(config.with_file_name(".env")).unwrap();
    assert!(environment.contains("TELEGRAM_BOT_TOKEN=existing-channel-token"));
    let token = environment
        .lines()
        .find(|line| line.starts_with("NEONEXUS_ASSISTANT_"))
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .trim_matches('"');
    assert!(token.starts_with("nnx_"));
    assert_eq!(
        request(
            &server,
            token,
            json!({"jsonrpc":"2.0","id":1,"method":"ping"})
        )
        .status(),
        200
    );
    let page = http
        .get(&format!("{}/assistants?edit=connection", server.base_url))
        .set("cookie", &cookie)
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    assert!(page.contains("Monitor only"));
    assert!(!page.contains(token));
    assert!(!page.contains("existing-channel-token"));
    assert_eq!(
        post_form_as(
            &http,
            &cookie,
            &format!("{}/assistants/connection/revoke", server.base_url),
            ""
        )
        .status(),
        303
    );
    assert_eq!(
        request(
            &server,
            token,
            json!({"jsonrpc":"2.0","id":1,"method":"ping"})
        )
        .status(),
        401
    );
}
