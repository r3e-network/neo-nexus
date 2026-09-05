use neo_nexus::assistants::hermes_config::{self, inject};
use std::fs;

#[test]
fn hermes_connection_preserves_channels_and_unrelated_mcp_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    let original = "model:\n  default: existing-model\nplatforms:\n  telegram:\n    enabled: true\nmcp_servers:\n  other:\n    command: existing-command\n  neonexus_test:\n    transport: http\n    tool_timeout: 75\n    headers:\n      X-Workspace: existing\n      authorization: obsolete\n";
    let environment = "# existing channels\r\nTELEGRAM_BOT_TOKEN=test-channel-secret\r\nTELEGRAM_ALLOWED_USERS=1234\r\nLLM_KEY='other-secret'\r\n";
    fs::write(&config, original).unwrap();
    fs::write(dir.path().join(".env"), environment).unwrap();
    let report = inject(
        &config,
        "http://127.0.0.1:8080/mcp",
        "neonexus_test",
        "NEONEXUS_ASSISTANT_TEST_TOKEN",
        "nnx_test-secret",
    )
    .unwrap();
    let changed = fs::read_to_string(&config).unwrap();
    let yaml: serde_yaml::Value = serde_yaml::from_str(&changed).unwrap();
    assert_eq!(
        yaml["platforms"]["telegram"]["enabled"].as_bool(),
        Some(true)
    );
    assert_eq!(yaml["model"]["default"].as_str(), Some("existing-model"));
    assert_eq!(
        yaml["mcp_servers"]["other"]["command"].as_str(),
        Some("existing-command")
    );
    let server = &yaml["mcp_servers"]["neonexus_test"];
    assert_eq!(server["tool_timeout"].as_u64(), Some(75));
    assert_eq!(server["headers"]["X-Workspace"].as_str(), Some("existing"));
    assert!(server["headers"].get("authorization").is_none());
    assert_eq!(
        server["headers"]["Authorization"].as_str(),
        Some("Bearer ${NEONEXUS_ASSISTANT_TEST_TOKEN}")
    );
    assert!(!changed.contains("nnx_test-secret"));
    let saved_env = fs::read_to_string(dir.path().join(".env")).unwrap();
    assert!(saved_env.starts_with(environment));
    assert!(saved_env.contains("NEONEXUS_ASSISTANT_TEST_TOKEN=nnx_test-secret\r\n"));
    assert_eq!(fs::read_to_string(&report.config_backup).unwrap(), original);
    assert_eq!(
        fs::read_to_string(report.environment_backup.as_ref().unwrap()).unwrap(),
        environment
    );
    assert!(!format!("{report:?}").contains("nnx_test-secret"));
    hermes_config::rollback(&report).unwrap();
    assert_eq!(fs::read_to_string(&config).unwrap(), original);
    assert_eq!(
        fs::read_to_string(dir.path().join(".env")).unwrap(),
        environment
    );
}

#[test]
fn hermes_connection_rotates_only_its_own_environment_key() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    fs::write(&config, "model: original\n").unwrap();
    inject(
        &config,
        "https://nexus.example/mcp",
        "neonexus_test",
        "NEONEXUS_ASSISTANT_TEST_TOKEN",
        "old-token",
    )
    .unwrap();
    inject(
        &config,
        "https://nexus.example/mcp",
        "neonexus_test",
        "NEONEXUS_ASSISTANT_TEST_TOKEN",
        "new-token",
    )
    .unwrap();
    let environment = fs::read_to_string(dir.path().join(".env")).unwrap();
    assert_eq!(
        environment
            .matches("NEONEXUS_ASSISTANT_TEST_TOKEN=")
            .count(),
        1
    );
    assert!(!environment.contains("old-token"));
    assert!(environment.contains("new-token"));
    assert_eq!(
        hermes_config::configured_endpoint(&config, "neonexus_test")
            .unwrap()
            .as_deref(),
        Some("https://nexus.example/mcp")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(dir.path().join(".env"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn invalid_mcp_endpoints_and_conflicts_leave_profile_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    let original = "mcp_servers:\n  neonexus_test:\n    command: custom-command\n";
    fs::write(&config, original).unwrap();
    for endpoint in [
        "http://remote.example/mcp",
        "https://user:secret@nexus.example/mcp",
        "https://nexus.example/mcp?token=secret",
        "https://nexus.example/",
        "file:///mcp",
    ] {
        assert!(hermes_config::validate(&config, endpoint).is_err());
    }
    assert!(inject(
        &config,
        "http://127.0.0.1:8080/mcp",
        "neonexus_test",
        "NEONEXUS_ASSISTANT_TEST_TOKEN",
        "new-token"
    )
    .is_err());
    assert_eq!(fs::read_to_string(&config).unwrap(), original);
    assert!(!dir.path().join(".env").exists());
    fs::write(&config, "key: [not-closed secret-do-not-echo").unwrap();
    let error = hermes_config::validate(&config, "http://127.0.0.1:8080/mcp")
        .unwrap_err()
        .to_string();
    assert!(!error.contains("secret-do-not-echo"));
}

#[test]
fn rollback_refuses_to_overwrite_newer_operator_edits() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    fs::write(&config, "model: original\n").unwrap();
    let report = inject(
        &config,
        "http://127.0.0.1:8080/mcp",
        "neonexus_test",
        "NEONEXUS_ASSISTANT_TEST_TOKEN",
        "new-token",
    )
    .unwrap();
    fs::write(&config, "model: operator-edit\n").unwrap();
    assert!(hermes_config::rollback(&report).is_err());
    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        "model: operator-edit\n"
    );
    assert_eq!(
        fs::read_to_string(&report.config_backup).unwrap(),
        "model: original\n"
    );
}

fn workspace(path: &std::path::Path) -> (neo_nexus::supervision::EngineState, String) {
    use neo_nexus::{
        agents::{AgentKind, AgentProfile},
        repository::Repository,
        supervision::EngineState,
        supervisor::ProcessSupervisor,
        types::{Network, NewNode, NodeType, StorageEngine},
    };
    use std::sync::{Arc, Mutex};
    let state = EngineState {
        repository: Repository::open(path.join("test.db")).unwrap(),
        data_dir: path.into(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        heartbeat: neo_nexus::supervision_heartbeat::SupervisionHeartbeat::new(),
        notifications: neo_nexus::supervision_heartbeat::SupervisionHeartbeat::new(),
    };
    let node = state
        .repository
        .create_node(NewNode {
            name: "managed".into(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: std::env::current_exe().unwrap(),
            args: vec![],
            runtime_version: "1".into(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 12332,
            p2p_port: 12333,
            ws_port: None,
        })
        .unwrap();
    fs::create_dir_all(path.join("hermes_cli")).unwrap();
    fs::write(
        path.join("hermes_cli/main.py"),
        "# fixture, never executed\n",
    )
    .unwrap();
    let python = path.join(if cfg!(windows) {
        "python.exe"
    } else {
        "python"
    });
    fs::hard_link(std::env::current_exe().unwrap(), &python)
        .or_else(|_| fs::copy(std::env::current_exe().unwrap(), &python).map(|_| ()))
        .unwrap();
    fs::write(path.join("config.yaml"), "model: existing\n").unwrap();
    neo_nexus::agents::save(
        &state,
        AgentProfile {
            id: "hermes".into(),
            name: "Hermes".into(),
            kind: AgentKind::Hermes,
            node_id: None,
            version: "fixture".into(),
            binary_path: python,
            working_dir: path.into(),
            args: vec![],
            config_path: Some(path.join("config.yaml")),
            health_url: None,
            auto_restart: false,
            binary_sha256: String::new(),
            config_sha256: None,
        },
    )
    .unwrap();
    (state, node.id)
}

#[test]
fn connection_updates_config_fingerprint_without_accepting_executable_drift() {
    use neo_nexus::assistants::{connect_hermes, AssistantDraft};
    let dir = tempfile::tempdir().unwrap();
    let (state, node_id) = workspace(dir.path());
    let before = state.repository.list_agents().unwrap().remove(0).profile;
    fs::write(
        dir.path().join("hermes_cli/main.py"),
        "# modified source still needs version review\n",
    )
    .unwrap();
    let connected = connect_hermes(
        &state,
        AssistantDraft {
            id: String::new(),
            name: "Node assistant".into(),
            agent_id: "hermes".into(),
            node_ids: vec![node_id],
            all_nodes: false,
            can_operate: false,
        },
        "http://127.0.0.1:8080/mcp",
    )
    .unwrap();
    let after = state.repository.list_agents().unwrap().remove(0).profile;
    assert_eq!(before.binary_sha256, after.binary_sha256);
    assert_ne!(before.config_sha256, after.config_sha256);
    assert!(connected.enabled && !connected.can_operate);
    assert!(fs::read_to_string(dir.path().join(".env"))
        .unwrap()
        .contains("nnx_"));
    assert!(!serde_json::to_string(&connected).unwrap().contains("nnx_"));
    assert!(neo_nexus::agents::start(&state, "hermes")
        .unwrap_err()
        .to_string()
        .contains("source changed"));
}

#[test]
fn preflight_conflict_leaves_grants_and_original_files_untouched() {
    use neo_nexus::assistants::{connect_hermes, AssistantDraft};
    let dir = tempfile::tempdir().unwrap();
    let (state, node_id) = workspace(dir.path());
    let conflicting = "mcp_servers:\n  neonexus_fixed:\n    command: operator-command\n";
    fs::write(dir.path().join("config.yaml"), conflicting).unwrap();
    assert!(connect_hermes(
        &state,
        AssistantDraft {
            id: "fixed".into(),
            name: "Node assistant".into(),
            agent_id: "hermes".into(),
            node_ids: vec![node_id],
            all_nodes: false,
            can_operate: false
        },
        "http://127.0.0.1:8080/mcp"
    )
    .is_err());
    assert!(state
        .repository
        .list_assistants()
        .unwrap()
        .iter()
        .all(|profile| !profile.enabled));
    assert_eq!(
        fs::read_to_string(dir.path().join("config.yaml")).unwrap(),
        conflicting
    );
    assert!(!dir.path().join(".env").exists());
}

#[test]
fn rebinding_a_grant_cannot_leave_another_hermes_installation_with_invalid_credentials() {
    use neo_nexus::assistants::{connect_hermes, AssistantDraft};
    let dir = tempfile::tempdir().unwrap();
    let (state, node_id) = workspace(dir.path());
    let draft = AssistantDraft {
        id: "fixed".into(),
        name: "Node assistant".into(),
        agent_id: "hermes".into(),
        node_ids: vec![node_id],
        all_nodes: false,
        can_operate: false,
    };
    connect_hermes(&state, draft.clone(), "http://127.0.0.1:8080/mcp").unwrap();
    let original_environment = fs::read_to_string(dir.path().join(".env")).unwrap();
    let mut other_agent = state.repository.list_agents().unwrap().remove(0).profile;
    other_agent.id = "another-hermes".into();
    neo_nexus::agents::save(&state, other_agent).unwrap();
    let mut rebind = draft;
    rebind.agent_id = "another-hermes".into();
    assert!(connect_hermes(&state, rebind, "http://127.0.0.1:8080/mcp")
        .unwrap_err()
        .to_string()
        .contains("separate connection"));
    let grant = state.repository.list_assistants().unwrap().remove(0);
    assert!(grant.enabled);
    assert_eq!(grant.agent_id, "hermes");
    assert_eq!(
        fs::read_to_string(dir.path().join(".env")).unwrap(),
        original_environment
    );
}

#[test]
fn database_failure_after_injection_restores_files_and_revokes_new_access() {
    use neo_nexus::assistants::{connect_hermes, AssistantDraft};
    let dir = tempfile::tempdir().unwrap();
    let (state, node_id) = workspace(dir.path());
    let original = fs::read_to_string(dir.path().join("config.yaml")).unwrap();
    let original_env = "TELEGRAM_BOT_TOKEN=existing-fixture-token\n";
    fs::write(dir.path().join(".env"), original_env).unwrap();
    // Fail only the final agent fingerprint write, after grant creation and
    // both configuration replacements have succeeded.
    let connection = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    connection.execute_batch("CREATE TRIGGER refuse_fingerprint BEFORE UPDATE ON managed_agents BEGIN SELECT RAISE(FAIL, 'fixture write failure'); END;").unwrap();
    let error = connect_hermes(
        &state,
        AssistantDraft {
            id: "rollback".into(),
            name: "Node assistant".into(),
            agent_id: "hermes".into(),
            node_ids: vec![node_id],
            all_nodes: false,
            can_operate: false,
        },
        "http://127.0.0.1:8080/mcp",
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("configuration restored and access revoked"));
    assert_eq!(
        fs::read_to_string(dir.path().join("config.yaml")).unwrap(),
        original
    );
    assert_eq!(
        fs::read_to_string(dir.path().join(".env")).unwrap(),
        original_env
    );
    let grants = state.repository.list_assistants().unwrap();
    assert_eq!(grants.len(), 1);
    assert!(!grants[0].enabled);
}
