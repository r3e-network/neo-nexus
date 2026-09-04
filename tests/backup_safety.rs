use neo_nexus::{
    backup::{
        EventBackup, WorkspaceBackup, WorkspaceBackupExporter, WorkspaceBackupImporter,
        WorkspaceSettingBackup,
    },
    repository::Repository,
    types::{Network, NewNode, NodeStatus, NodeType},
    wallet::NeoWalletProfile,
};

fn node(repository: &Repository, node_type: NodeType, args: Vec<String>) -> String {
    repository
        .create_node(NewNode {
            name: "restore fixture".into(),
            node_type,
            network: Network::Testnet,
            binary_path: std::env::current_exe().unwrap(),
            args,
            runtime_version: "test".into(),
            storage_engine: node_type.default_storage_engine(),
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap()
        .id
}

fn wallet() -> NeoWalletProfile {
    NeoWalletProfile {
        id: "validator-wallet".into(),
        label: "Validator wallet".into(),
        source_path: "/wallets/validator.json".into(),
        wallet_version: Some("3.0".into()),
        primary_address: "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq".into(),
        contract_public_keys: vec![
            "036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0".into(),
        ],
        wallet_sha256: "e".repeat(64),
        account_count: 1,
        encrypted_account_count: 1,
        default_account_count: 1,
        watch_only_account_count: 0,
        validated_at_unix: 1,
        last_used_at_unix: None,
    }
}

fn snapshot(repository: &Repository) -> WorkspaceBackup {
    WorkspaceBackupExporter::snapshot(repository, "test", 123).unwrap()
}

#[test]
fn importing_over_running_starting_or_unresolved_pid_preserves_everything() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("test.db")).unwrap();
    let id = node(&repository, NodeType::NeoRs, vec![]);
    let mut backup = snapshot(&repository);
    backup.nodes[0].name = "replacement".into();
    backup.workspace_settings.push(WorkspaceSettingBackup {
        key: "watchdog.enabled".into(),
        value: "true".into(),
    });
    for (status, pid) in [
        (NodeStatus::Running, Some(123)),
        (NodeStatus::Starting, None),
        (NodeStatus::Crashed, Some(456)),
    ] {
        repository.update_node_status(&id, status, pid).unwrap();
        let before = repository.list_nodes().unwrap();
        assert!(
            WorkspaceBackupImporter::validate_target(&repository, &backup)
                .unwrap_err()
                .to_string()
                .contains("stop node")
        );
        assert!(WorkspaceBackupImporter::import(&repository, &backup)
            .unwrap_err()
            .to_string()
            .contains("stop node"));
        assert_eq!(repository.list_nodes().unwrap(), before);
        assert!(repository
            .list_workspace_settings_for_backup()
            .unwrap()
            .is_empty());
    }
}

#[test]
fn import_failure_rolls_back_settings_profiles_and_earlier_node_changes() {
    let dir = tempfile::tempdir().unwrap();
    let source = Repository::open(dir.path().join("source.db")).unwrap();
    let target_path = dir.path().join("target.db");
    let target = Repository::open(&target_path).unwrap();
    let id = node(&source, NodeType::NeoRs, vec![]);
    let mut backup = snapshot(&source);
    WorkspaceBackupImporter::import(&target, &backup).unwrap();
    source.upsert_neo_wallet_profile(&wallet()).unwrap();
    source
        .set_node_wallet(&id, Some("validator-wallet"))
        .unwrap();
    backup = snapshot(&source);
    backup.nodes[0].name = "replacement".into();
    backup.workspace_settings.push(WorkspaceSettingBackup {
        key: "watchdog.enabled".into(),
        value: "true".into(),
    });
    backup.events.push(EventBackup {
        id: 1,
        occurred_at_unix: 123,
        node_id: Some(id.clone()),
        node_name: Some("fixture".into()),
        kind: neo_nexus::events::EventKind::NodeStarted.to_string(),
        severity: "info".into(),
        message: "fault injection".into(),
    });
    rusqlite::Connection::open(&target_path).unwrap().execute_batch(
        "CREATE TRIGGER fail_restore_event BEFORE INSERT ON runtime_events BEGIN SELECT RAISE(ABORT, 'restore fault fixture'); END;"
    ).unwrap();
    let error = WorkspaceBackupImporter::import(&target, &backup).unwrap_err();
    assert!(format!("{error:#}").contains("restore fault fixture"));
    assert_eq!(target.list_nodes().unwrap()[0].name, "restore fixture");
    assert!(target.list_neo_wallet_profiles().unwrap().is_empty());
    assert!(target.load_node_wallet(&id).unwrap().is_none());
    assert!(target
        .list_workspace_settings_for_backup()
        .unwrap()
        .is_empty());
    assert!(target.list_recent_events(10).unwrap().is_empty());
}

#[test]
fn wallet_references_must_be_complete_and_cannot_rebind_an_existing_identity() {
    let dir = tempfile::tempdir().unwrap();
    let source = Repository::open(dir.path().join("source.db")).unwrap();
    let target = Repository::open(dir.path().join("target.db")).unwrap();
    let id = node(&source, NodeType::NeoGo, vec![]);
    source
        .set_node_wallet(&id, Some("validator-wallet"))
        .unwrap();
    assert!(WorkspaceBackupExporter::snapshot(&source, "test", 123).is_err());
    source.upsert_neo_wallet_profile(&wallet()).unwrap();
    let mut backup = snapshot(&source);
    let profiles = std::mem::take(&mut backup.neo_wallet_profiles);
    assert!(WorkspaceBackupImporter::validate(&backup)
        .unwrap_err()
        .to_string()
        .contains("wallet profile"));
    backup.neo_wallet_profiles = profiles;
    let mut existing = wallet();
    existing.wallet_sha256 = "f".repeat(64);
    target.upsert_neo_wallet_profile(&existing).unwrap();
    assert!(WorkspaceBackupImporter::import(&target, &backup)
        .unwrap_err()
        .to_string()
        .contains("local signing identity"));
    assert_eq!(
        target.list_neo_wallet_profiles().unwrap()[0].wallet_sha256,
        "f".repeat(64)
    );
    assert!(target.list_nodes().unwrap().is_empty());
}

#[test]
fn raw_credential_arguments_are_rejected_but_native_file_references_round_trip() {
    for (node_type, args) in [
        (
            NodeType::NeoXGeth,
            vec!["--nodekeyhex", "literal-private-key"],
        ),
        (NodeType::NeoRs, vec!["--password=literal-password"]),
        (
            NodeType::NeoGo,
            vec!["--endpoint", "https://user:password@example.test"],
        ),
        (
            NodeType::NeoRs,
            vec!["--endpoint=https://example.test?%74oken=literal-token"],
        ),
        (
            NodeType::NeoRs,
            vec!["--endpoint", "https://example.test?%74oken=literal-token"],
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let repository = Repository::open(dir.path().join("test.db")).unwrap();
        node(
            &repository,
            node_type,
            args.iter().map(|arg| (*arg).into()).collect(),
        );
        let error = WorkspaceBackupExporter::snapshot(&repository, "test", 123).unwrap_err();
        assert!(!error.to_string().contains("literal-"));
    }
    for (node_type, args) in [
        (
            NodeType::NeoXGeth,
            vec!["--password", "/wallets/password.txt"],
        ),
        (
            NodeType::NeoXGeth,
            vec!["--authrpc.jwtsecret", "/wallets/jwt-secret.txt"],
        ),
        (
            NodeType::NeoRs,
            vec!["--password-file=/wallets/password.txt"],
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let repository = Repository::open(dir.path().join("test.db")).unwrap();
        let args: Vec<String> = args.iter().map(|arg| (*arg).into()).collect();
        node(&repository, node_type, args.clone());
        let mut backup = snapshot(&repository);
        assert_eq!(backup.nodes[0].args, args);
        WorkspaceBackupImporter::validate(&backup).unwrap();
        backup.nodes[0].args = vec!["--private-key=literal-key".into()];
        assert!(WorkspaceBackupImporter::validate(&backup).is_err());
    }
}

#[test]
fn restored_event_messages_are_redacted_and_operational_settings_stay_excluded() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("test.db")).unwrap();
    let mut backup = snapshot(&repository);
    backup.events.push(EventBackup {
        id: 1,
        occurred_at_unix: 123,
        node_id: None,
        node_name: None,
        kind: neo_nexus::events::EventKind::NodeStarted.to_string(),
        severity: "info".into(),
        message: "password=imported-secret".into(),
    });
    WorkspaceBackupImporter::import(&repository, &backup).unwrap();
    assert!(!repository.list_recent_events(10).unwrap()[0]
        .message
        .contains("imported-secret"));
    for key in [
        "resource_health.snapshot",
        "assistant.token",
        "signer.auth_token",
        "alert_routing.webhook_url",
    ] {
        backup.workspace_settings = vec![WorkspaceSettingBackup {
            key: key.into(),
            value: "secret".into(),
        }];
        assert!(WorkspaceBackupImporter::validate(&backup).is_err(), "{key}");
    }
}

#[test]
fn backup_resource_policy_is_validated_before_changes() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("test.db")).unwrap();
    let mut backup = snapshot(&repository);
    backup.workspace_settings.push(WorkspaceSettingBackup {
        key: "resource_health.policy".into(),
        value: r#"{"memory_critical_percent":100}"#.into(),
    });
    assert!(WorkspaceBackupImporter::validate(&backup).is_err());
    backup.workspace_settings[0].value =
        serde_json::to_string(&neo_nexus::resource_health::ResourcePolicy::default()).unwrap();
    WorkspaceBackupImporter::import(&repository, &backup).unwrap();
}

#[test]
fn pending_recovery_blocks_restore_and_inactive_budget_is_cleared() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let repository = Repository::open(&path).unwrap();
    let id = node(&repository, NodeType::NeoRs, vec![]);
    let backup = snapshot(&repository);
    repository
        .update_node_status(&id, NodeStatus::Crashed, None)
        .unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    let key = format!("watchdog.recovery.{id}");
    for (deadline, claim) in [
        (Some(123_u64), None),
        (None, Some(uuid::Uuid::new_v4().to_string())),
    ] {
        connection.execute("INSERT INTO workspace_settings(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            rusqlite::params![key, serde_json::json!({"version":1, "attempts":1, "next_attempt_at_unix_ms":deadline, "claim":claim, "exhausted":false}).to_string()]).unwrap();
        assert!(WorkspaceBackupImporter::import(&repository, &backup)
            .unwrap_err()
            .to_string()
            .contains("automatic recovery"));
        assert_eq!(
            repository.list_nodes().unwrap()[0].status,
            NodeStatus::Crashed
        );
    }
    connection.execute("UPDATE workspace_settings SET value=?1 WHERE key=?2",
        rusqlite::params![serde_json::json!({"version":1, "attempts":3, "next_attempt_at_unix_ms":null, "claim":null, "exhausted":true}).to_string(), key]).unwrap();
    WorkspaceBackupImporter::import(&repository, &backup).unwrap();
    assert_eq!(
        repository.list_nodes().unwrap()[0].status,
        NodeStatus::Stopped
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM workspace_settings WHERE key=?1",
                [&key],
                |row| row.get::<_, usize>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn associated_running_agent_blocks_node_restore_without_touching_its_record() {
    use neo_nexus::agents::{AgentKind, AgentProfile, AgentRecord, AgentStatus};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let repository = Repository::open(&path).unwrap();
    let id = node(&repository, NodeType::NeoRs, vec![]);
    let backup = snapshot(&repository);
    let record = AgentRecord {
        profile: AgentProfile {
            id: "sidecar".into(),
            name: "sidecar".into(),
            kind: AgentKind::Sidecar,
            node_id: Some(id.clone()),
            version: "test".into(),
            binary_path: std::env::current_exe().unwrap(),
            working_dir: dir.path().into(),
            args: vec![],
            config_path: None,
            health_url: None,
            auto_restart: true,
            binary_sha256: String::new(),
            config_sha256: None,
        },
        status: AgentStatus::Running,
        pid: Some(987654),
        process_started_at: Some(42),
        desired_running: true,
        restart_attempts: 1,
        restart_after: None,
        healthy: Some(true),
        last_health_at: 123,
    };
    let encoded = serde_json::to_string(&record).unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "INSERT INTO managed_agents(id,record) VALUES ('sidecar',?1)",
            [&encoded],
        )
        .unwrap();
    assert!(WorkspaceBackupImporter::import(&repository, &backup)
        .unwrap_err()
        .to_string()
        .contains("associated with node"));
    assert_eq!(
        connection
            .query_row("SELECT record FROM managed_agents", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        encoded
    );
    let mut stopped = repository.list_nodes().unwrap().remove(0);
    stopped.name = "direct restore".into();
    assert!(repository.restore_node_with_plugins(&stopped, &[]).is_err());
    assert_eq!(repository.list_nodes().unwrap()[0].name, "restore fixture");
}

#[test]
fn catalog_credentials_are_blocked_on_export_and_import_and_trust_changes_require_review() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use neo_nexus::runtime::{RuntimeCatalogProfile, RuntimeSignerProfile};
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("test.db")).unwrap();
    let mut profile = RuntimeCatalogProfile {
        id: "catalog".into(),
        label: "Catalog".into(),
        source: "https://example.test/catalog.json".into(),
        signature_source: Some("https://example.test/catalog.sig".into()),
        ed25519_public_key: Some(STANDARD.encode([1; 32])),
        max_bytes: 1024,
        enabled: true,
        last_loaded_at_unix: None,
        last_signature_verified: None,
        last_bytes: None,
    };
    repository.upsert_runtime_catalog_profile(&profile).unwrap();
    let clean = snapshot(&repository);
    for source in [
        "https://user:private-credential@example.test/catalog.json",
        "https://example.test/catalog.json?sig=private-credential",
        "https://example.test/catalog.json?%74oken=private-credential",
    ] {
        profile.source = source.into();
        repository.upsert_runtime_catalog_profile(&profile).unwrap();
        let error = WorkspaceBackupExporter::snapshot(&repository, "test", 123).unwrap_err();
        assert!(!error.to_string().contains("private-credential"));
        let mut backup = clean.clone();
        backup.runtime_catalog_profiles[0].source = source.into();
        assert!(WorkspaceBackupImporter::validate(&backup).is_err());
    }
    profile.source = "https://different.example.test/catalog.json".into();
    repository.upsert_runtime_catalog_profile(&profile).unwrap();
    assert!(
        WorkspaceBackupImporter::validate_target(&repository, &clean)
            .unwrap_err()
            .to_string()
            .contains("local source or trusted key")
    );
    let signer = RuntimeSignerProfile {
        id: "release-key".into(),
        label: "Release key".into(),
        ed25519_public_key: STANDARD.encode([1; 32]),
        enabled: true,
        created_at_unix: 1,
        last_used_at_unix: None,
    };
    repository.upsert_runtime_signer_profile(&signer).unwrap();
    let mut backup = snapshot(&repository);
    backup.runtime_signer_profiles[0].ed25519_public_key = STANDARD.encode(
        ed25519_dalek::SigningKey::from_bytes(&[2; 32])
            .verifying_key()
            .to_bytes(),
    );
    assert!(WorkspaceBackupImporter::import(&repository, &backup)
        .unwrap_err()
        .to_string()
        .contains("local trusted key"));
}

#[test]
fn repeated_exports_preserve_previous_files_and_discovery_ignores_partial_writes() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("test.db")).unwrap();
    node(&repository, NodeType::NeoRs, vec![]);
    let output = dir.path().join("backups");
    WorkspaceBackupExporter::write(&repository, &output, "first").unwrap();
    let first = WorkspaceBackupImporter::latest_backup_path(&output)
        .unwrap()
        .unwrap();
    let first_bytes = std::fs::read(&first).unwrap();
    WorkspaceBackupExporter::write(&repository, &output, "second").unwrap();
    assert_eq!(std::fs::read(&first).unwrap(), first_bytes);
    let files: Vec<_> = std::fs::read_dir(&output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(files.len(), 2);
    for file in files {
        WorkspaceBackupImporter::validate_path(file).unwrap();
    }
    let latest = WorkspaceBackupImporter::latest_backup_path(&output)
        .unwrap()
        .unwrap();
    std::fs::write(
        output.join(".neonexus-backup-9999999999-pending.tmp"),
        "partial",
    )
    .unwrap();
    std::fs::create_dir(output.join("neonexus-backup-9999999999.json")).unwrap();
    assert_eq!(
        WorkspaceBackupImporter::latest_backup_path(&output)
            .unwrap()
            .unwrap(),
        latest
    );
    // The original timestamp-only naming remains discoverable.
    std::fs::write(output.join("neonexus-backup-9999999998.json"), &first_bytes).unwrap();
    assert_eq!(
        WorkspaceBackupImporter::latest_backup_path(&output)
            .unwrap()
            .unwrap()
            .file_name()
            .unwrap(),
        "neonexus-backup-9999999998.json"
    );
}
