use neo_nexus::backup::WorkspaceBackupImport;

use super::*;

pub(super) fn assert_first_import(
    target: &Repository,
    imported: &WorkspaceBackupImport,
    node_id: &str,
) {
    assert_eq!(imported.created_nodes, 1);
    assert_eq!(imported.updated_nodes, 0);
    assert_eq!(imported.plugin_state_count, 2);
    assert_eq!(imported.plugin_installation_count, 1);
    assert_eq!(imported.workspace_setting_count, 6);
    assert_eq!(imported.remote_server_count, 1);
    assert_eq!(imported.runtime_catalog_profile_count, 1);
    assert_eq!(imported.runtime_signer_profile_count, 1);
    assert_eq!(imported.neo_wallet_profile_count, 1);
    assert_eq!(imported.fast_sync_snapshot_count, 1);
    assert_eq!(imported.event_count, 1);

    let restored = target.list_nodes().unwrap();
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].id, node_id);
    assert_eq!(restored[0].name, "restore source");
    assert_eq!(restored[0].status, NodeStatus::Stopped);
    assert_eq!(restored[0].pid, None);
    assert_eq!(target.list_plugin_states(node_id).unwrap().len(), 2);
    let restored_plugins = target.list_plugin_installations(node_id).unwrap();
    assert_eq!(restored_plugins.len(), 1);
    assert_eq!(restored_plugins[0].plugin_id, PluginId::RpcServer);
    assert_eq!(restored_plugins[0].sha256, "d".repeat(64));
    assert_eq!(
        target.load_watchdog_policy().unwrap(),
        RestartPolicy::with_enabled(false, 9, Duration::from_secs(7), Duration::from_secs(70))
    );
    assert_eq!(
        target.load_rpc_health_monitor_policy().unwrap(),
        RpcHealthMonitorPolicy {
            enabled: false,
            interval_seconds: 300
        }
    );
    let runtime_catalogs = target.list_runtime_catalog_profiles().unwrap();
    assert_eq!(runtime_catalogs.len(), 1);
    assert_eq!(runtime_catalogs[0].id, "restore-runtime-catalog");
    assert_eq!(runtime_catalogs[0].last_bytes, Some(128));
    let remote_servers = target.list_remote_servers().unwrap();
    assert_eq!(remote_servers.len(), 1);
    assert_eq!(remote_servers[0].name, "Restore Remote");
    assert_eq!(
        remote_servers[0].base_url,
        "https://restore.example.com/neo"
    );
    assert!(!remote_servers[0].enabled);
    let runtime_signers = target.list_runtime_signer_profiles().unwrap();
    assert_eq!(runtime_signers.len(), 1);
    assert_eq!(runtime_signers[0].id, "restore-runtime-signer");
    let wallet_profiles = target.list_neo_wallet_profiles().unwrap();
    assert_eq!(wallet_profiles.len(), 1);
    assert_eq!(wallet_profiles[0].id, "restore-wallet-profile");
    assert_eq!(
        wallet_profiles[0].primary_address,
        "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq"
    );
    let snapshots = target.list_fast_sync_snapshots().unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].id, "restore-snapshot");
    assert_eq!(snapshots[0].cached_path, None);
    assert_eq!(snapshots[0].verified_sha256, None);
    let restored_events = target.list_recent_events(10).unwrap();
    assert_eq!(restored_events.len(), 1);
    assert_eq!(restored_events[0].occurred_at_unix, 1_800_000_010);
    assert_eq!(restored_events[0].node_id.as_deref(), Some(node_id));
    assert_eq!(
        restored_events[0].node_name.as_deref(),
        Some("restore source")
    );
    assert_eq!(restored_events[0].kind, EventKind::BackupExported);
    assert_eq!(restored_events[0].severity, EventSeverity::Info);
    assert_eq!(restored_events[0].message, "source backup");
}

pub(super) fn assert_second_import(target: &Repository, imported_again: &WorkspaceBackupImport) {
    assert_eq!(imported_again.created_nodes, 0);
    assert_eq!(imported_again.updated_nodes, 1);
    assert_eq!(imported_again.plugin_installation_count, 1);
    assert_eq!(imported_again.remote_server_count, 1);
    assert_eq!(imported_again.runtime_catalog_profile_count, 1);
    assert_eq!(imported_again.runtime_signer_profile_count, 1);
    assert_eq!(imported_again.fast_sync_snapshot_count, 1);
    assert_eq!(imported_again.event_count, 0);
    assert_eq!(target.list_recent_events(10).unwrap().len(), 1);
}
