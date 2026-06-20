use std::path::Path;

use super::*;

pub(super) fn populate_backup_source(repo: &Repository, workspace_root: &Path) {
    repo.save_rpc_health_monitor_policy(RpcHealthMonitorPolicy {
        enabled: false,
        interval_seconds: 120,
    })
    .unwrap();
    repo.upsert_runtime_catalog_profile(&RuntimeCatalogProfile {
        id: "runtime-catalog-backup".to_string(),
        label: "Runtime Catalog Backup".to_string(),
        source: workspace_root
            .join("runtime-catalog.json")
            .display()
            .to_string(),
        signature_source: None,
        ed25519_public_key: None,
        max_bytes: 1_048_576,
        enabled: true,
        last_loaded_at_unix: Some(1_800_000_003),
        last_signature_verified: Some(true),
        last_bytes: Some(512),
    })
    .unwrap();
    let signing_key = SigningKey::from_bytes(&[41u8; 32]);
    repo.upsert_runtime_signer_profile(&RuntimeSignerProfile {
        id: "runtime-signer-backup".to_string(),
        label: "Runtime Signer Backup".to_string(),
        ed25519_public_key: BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes()),
        enabled: true,
        created_at_unix: 1_800_000_004,
        last_used_at_unix: Some(1_800_000_005),
    })
    .unwrap();
    repo.upsert_neo_wallet_profile(&NeoWalletProfile {
        id: "validator-wallet-backup".to_string(),
        label: "Validator Wallet Backup".to_string(),
        source_path: workspace_root
            .join("wallets/validator.wallet.json")
            .display()
            .to_string(),
        wallet_version: Some("3.0".to_string()),
        primary_address: "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq".to_string(),
        contract_public_keys: vec![
            "036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0".to_string(),
        ],
        wallet_sha256: "b".repeat(64),
        account_count: 1,
        encrypted_account_count: 1,
        default_account_count: 1,
        watch_only_account_count: 0,
        validated_at_unix: 1_800_000_006,
        last_used_at_unix: Some(1_800_000_007),
    })
    .unwrap();
    repo.upsert_fast_sync_snapshot(NewFastSyncSnapshot {
        id: "snapshot-backup".to_string(),
        label: "Snapshot Backup".to_string(),
        network: Network::Testnet,
        node_type: NodeType::NeoRs,
        source_path: workspace_root.join("snapshots/source.acc"),
        source_url: Some("https://snapshots.example.com/source.acc".to_string()),
        download_file_name: Some("source.acc".to_string()),
        download_max_bytes: 2_097_152,
        expected_sha256: "a".repeat(64),
    })
    .unwrap();
    repo.create_remote_server(NewRemoteServerProfile {
        name: "Remote Backup".to_string(),
        base_url: "nexus.example.com/ops/".to_string(),
        description: "Read-only public federation endpoint".to_string(),
        enabled: true,
    })
    .unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "backup node".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Testnet,
            binary_path: PathBuf::from("/usr/local/bin/neo-cli"),
            args: vec!["--config".to_string(), "config.json".to_string()],
            runtime_version: "v3.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 30332,
            p2p_port: 30333,
            ws_port: Some(30334),
        })
        .unwrap()
        .id;
    repo.set_plugin_enabled(&node_id, PluginId::RpcServer, true)
        .unwrap();
    repo.set_plugin_enabled(&node_id, PluginId::RocksDbStore, true)
        .unwrap();
    repo.upsert_plugin_installation(&PluginInstallation {
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
        sha256: "c".repeat(64),
        package_bytes: 512,
        installed_files: 2,
        expanded_bytes: 1024,
        installed_at_unix: 1_800_000_002,
    })
    .unwrap();
    repo.record_event_at(
        NewRuntimeEvent {
            node_id: Some(node_id.clone()),
            node_name: Some("backup node".to_string()),
            kind: EventKind::BackupExported,
            severity: EventSeverity::Info,
            message: "test event Authorization: Bearer backup-token api_key:abc123 seed=raw"
                .to_string(),
        },
        1_800_000_001,
    )
    .unwrap();
}
