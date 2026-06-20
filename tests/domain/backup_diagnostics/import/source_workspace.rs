use std::path::Path;

use neo_nexus::backup::WorkspaceBackup;

use super::*;

pub(super) fn backup_with_full_workspace(workspace_root: &Path) -> (WorkspaceBackup, String) {
    let source = Repository::open(workspace_root.join("source.db")).unwrap();
    source
        .save_watchdog_policy(RestartPolicy::with_enabled(
            false,
            9,
            Duration::from_secs(7),
            Duration::from_secs(70),
        ))
        .unwrap();
    source
        .save_rpc_health_monitor_policy(RpcHealthMonitorPolicy {
            enabled: false,
            interval_seconds: 300,
        })
        .unwrap();
    source
        .upsert_runtime_catalog_profile(&RuntimeCatalogProfile {
            id: "restore-runtime-catalog".to_string(),
            label: "Restore Runtime Catalog".to_string(),
            source: workspace_root
                .join("restore-runtime-catalog.json")
                .display()
                .to_string(),
            signature_source: None,
            ed25519_public_key: None,
            max_bytes: 1_048_576,
            enabled: true,
            last_loaded_at_unix: Some(1_800_000_012),
            last_signature_verified: None,
            last_bytes: Some(128),
        })
        .unwrap();
    let signing_key = SigningKey::from_bytes(&[42u8; 32]);
    source
        .upsert_runtime_signer_profile(&RuntimeSignerProfile {
            id: "restore-runtime-signer".to_string(),
            label: "Restore Runtime Signer".to_string(),
            ed25519_public_key: BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes()),
            enabled: true,
            created_at_unix: 1_800_000_013,
            last_used_at_unix: Some(1_800_000_014),
        })
        .unwrap();
    source
        .upsert_neo_wallet_profile(&NeoWalletProfile {
            id: "restore-wallet-profile".to_string(),
            label: "Restore Wallet Profile".to_string(),
            source_path: workspace_root
                .join("wallets/restore.wallet.json")
                .display()
                .to_string(),
            wallet_version: Some("3.0".to_string()),
            primary_address: "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq".to_string(),
            contract_public_keys: vec![
                "036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0".to_string(),
            ],
            wallet_sha256: "e".repeat(64),
            account_count: 1,
            encrypted_account_count: 1,
            default_account_count: 1,
            watch_only_account_count: 0,
            validated_at_unix: 1_800_000_015,
            last_used_at_unix: Some(1_800_000_016),
        })
        .unwrap();
    source
        .upsert_fast_sync_snapshot(NewFastSyncSnapshot {
            id: "restore-snapshot".to_string(),
            label: "Restore Snapshot".to_string(),
            network: Network::Private,
            node_type: NodeType::NeoRs,
            source_path: workspace_root.join("restore-snapshot.acc"),
            source_url: Some("https://snapshots.example.com/restore.acc".to_string()),
            download_file_name: Some("restore.acc".to_string()),
            download_max_bytes: 4_194_304,
            expected_sha256: "b".repeat(64),
        })
        .unwrap();
    source
        .create_remote_server(NewRemoteServerProfile {
            name: "Restore Remote".to_string(),
            base_url: "restore.example.com/neo/".to_string(),
            description: "Remote restore endpoint".to_string(),
            enabled: false,
        })
        .unwrap();
    let node_id = source
        .create_node(NewNode {
            name: "restore source".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: PathBuf::from("/usr/local/bin/neo-cli"),
            args: vec!["--config".to_string(), "restore.json".to_string()],
            runtime_version: "v3.8.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap()
        .id;
    source
        .update_node_status(&node_id, NodeStatus::Running, Some(4242))
        .unwrap();
    source
        .set_plugin_enabled(&node_id, PluginId::RpcServer, true)
        .unwrap();
    source
        .set_plugin_enabled(&node_id, PluginId::LevelDbStore, true)
        .unwrap();
    source
        .upsert_plugin_installation(&PluginInstallation {
            node_id: node_id.clone(),
            plugin_id: PluginId::RpcServer,
            installed_path: workspace_root
                .join("nodes")
                .join(&node_id)
                .join("Plugins/RpcServer"),
            manifest_path: workspace_root
                .join("nodes")
                .join(&node_id)
                .join("Plugins/RpcServer/.neonexus/manifest.json"),
            source_path: workspace_root.join("packages/rpc-server.zip"),
            sha256: "d".repeat(64),
            package_bytes: 256,
            installed_files: 1,
            expanded_bytes: 128,
            installed_at_unix: 1_800_000_009,
        })
        .unwrap();
    source
        .record_event_at(
            NewRuntimeEvent {
                node_id: Some(node_id.clone()),
                node_name: Some("restore source".to_string()),
                kind: EventKind::BackupExported,
                severity: EventSeverity::Info,
                message: "source backup".to_string(),
            },
            1_800_000_010,
        )
        .unwrap();
    let backup = WorkspaceBackupExporter::snapshot(&source, "2.5.3-test", 1_800_000_011).unwrap();
    (backup, node_id)
}
