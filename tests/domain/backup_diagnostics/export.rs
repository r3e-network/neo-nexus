use super::*;

#[path = "export/source_workspace.rs"]
mod source_workspace;

#[test]
fn workspace_backup_exports_nodes_and_plugin_state() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    source_workspace::populate_backup_source(&repo, temp_dir.path());

    let backup = WorkspaceBackupExporter::snapshot(&repo, "2.5.3-test", 1_800_000_000).unwrap();

    assert_eq!(backup.schema_version, 7);
    assert_eq!(backup.application, "NeoNexus");
    assert_eq!(backup.application_version, "2.5.3-test");
    assert_eq!(backup.exported_at_unix, 1_800_000_000);
    assert_eq!(backup.workspace_settings.len(), 2);
    assert!(backup
        .workspace_settings
        .iter()
        .any(|setting| setting.key == "rpc_health_monitor.enabled" && setting.value == "false"));
    assert_eq!(backup.runtime_catalog_profiles.len(), 1);
    assert_eq!(
        backup.runtime_catalog_profiles[0].id,
        "runtime-catalog-backup"
    );
    assert_eq!(backup.runtime_signer_profiles.len(), 1);
    assert_eq!(
        backup.runtime_signer_profiles[0].id,
        "runtime-signer-backup"
    );
    assert_eq!(backup.neo_wallet_profiles.len(), 1);
    assert_eq!(backup.neo_wallet_profiles[0].id, "validator-wallet-backup");
    assert_eq!(
        backup.neo_wallet_profiles[0].primary_address,
        "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq"
    );
    assert_eq!(backup.fast_sync_snapshots.len(), 1);
    assert_eq!(backup.fast_sync_snapshots[0].id, "snapshot-backup");
    assert_eq!(backup.remote_servers.len(), 1);
    assert_eq!(backup.remote_servers[0].name, "Remote Backup");
    assert_eq!(
        backup.remote_servers[0].base_url,
        "https://nexus.example.com/ops"
    );
    assert_eq!(backup.nodes.len(), 1);
    assert_eq!(backup.nodes[0].name, "backup node");
    assert_eq!(backup.nodes[0].plugins.len(), 2);
    assert_eq!(backup.nodes[0].plugin_installations.len(), 1);
    assert_eq!(backup.events.len(), 1);
    assert_eq!(backup.events[0].kind, "backup-exported");
    assert!(backup.events[0]
        .message
        .contains("Authorization:<redacted>"));
    assert!(backup.events[0].message.contains("api_key:<redacted>"));
    assert!(backup.events[0].message.contains("seed=<redacted>"));
    assert!(!backup.events[0].message.contains("backup-token"));
    assert!(!backup.events[0].message.contains("abc123"));
    assert!(!backup.events[0].message.contains("seed=raw"));

    let export =
        WorkspaceBackupExporter::write(&repo, temp_dir.path().join("backups"), "2.5.3-test")
            .unwrap();
    let text = std::fs::read_to_string(&export.path).unwrap();
    let latest = WorkspaceBackupImporter::latest_backup_path(temp_dir.path().join("backups"))
        .unwrap()
        .unwrap();
    let parsed = WorkspaceBackupImporter::read(&latest).unwrap();
    let validation = WorkspaceBackupImporter::validate(&parsed).unwrap();

    assert_eq!(export.node_count, 1);
    assert_eq!(export.plugin_state_count, 2);
    assert_eq!(export.plugin_installation_count, 1);
    assert_eq!(export.workspace_setting_count, 2);
    assert_eq!(export.remote_server_count, 1);
    assert_eq!(export.runtime_catalog_profile_count, 1);
    assert_eq!(export.runtime_signer_profile_count, 1);
    assert_eq!(export.neo_wallet_profile_count, 1);
    assert_eq!(export.fast_sync_snapshot_count, 1);
    assert_eq!(export.event_count, 1);
    assert_eq!(latest, export.path);
    assert_eq!(parsed.nodes.len(), 1);
    assert_eq!(parsed.workspace_settings.len(), 2);
    assert_eq!(parsed.remote_servers.len(), 1);
    assert_eq!(parsed.runtime_catalog_profiles.len(), 1);
    assert_eq!(parsed.runtime_signer_profiles.len(), 1);
    assert_eq!(parsed.neo_wallet_profiles.len(), 1);
    assert_eq!(parsed.fast_sync_snapshots.len(), 1);
    assert_eq!(validation.source_path, None);
    assert_eq!(validation.schema_version, 7);
    assert_eq!(validation.node_count, 1);
    assert_eq!(validation.plugin_state_count, 2);
    assert_eq!(validation.plugin_installation_count, 1);
    assert_eq!(validation.workspace_setting_count, 2);
    assert_eq!(validation.remote_server_count, 1);
    assert_eq!(validation.runtime_catalog_profile_count, 1);
    assert_eq!(validation.runtime_signer_profile_count, 1);
    assert_eq!(validation.neo_wallet_profile_count, 1);
    assert_eq!(validation.fast_sync_snapshot_count, 1);
    assert_eq!(validation.event_count, 1);
    assert!(text.contains("\"application\": \"NeoNexus\""));
    assert!(text.contains("\"workspace_settings\""));
    assert!(text.contains("\"remote_servers\""));
    assert!(text.contains("\"runtime_catalog_profiles\""));
    assert!(text.contains("\"runtime_signer_profiles\""));
    assert!(text.contains("\"neo_wallet_profiles\""));
    assert!(text.contains("\"wallet_sha256\": \"bbbb"));
    assert!(text.contains("\"fast_sync_snapshots\""));
    assert!(text.contains("\"key\": \"rpc_health_monitor.interval_seconds\""));
    assert!(text.contains("\"plugin_id\": \"RpcServer\""));
    assert!(text.contains("\"plugin_installations\""));
    assert!(text.contains("\"kind\": \"backup-exported\""));
    assert!(text.contains("Authorization:<redacted>"));
    assert!(!text.contains("backup-token"));
    assert!(!text.contains("abc123"));
    assert!(!text.contains("seed=raw"));
    assert!(!text.contains("6PYW"));
    assert!(!text.contains("password"));
}
