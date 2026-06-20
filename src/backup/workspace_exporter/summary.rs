use super::{writer::WrittenBackup, WorkspaceBackup, WorkspaceBackupExport};

pub(super) fn backup_export_summary(
    backup: WorkspaceBackup,
    written: WrittenBackup,
) -> WorkspaceBackupExport {
    WorkspaceBackupExport {
        path: written.path,
        schema_version: backup.schema_version,
        application_version: backup.application_version,
        exported_at_unix: backup.exported_at_unix,
        bytes_written: written.bytes_written,
        node_count: backup.nodes.len(),
        plugin_state_count: backup.nodes.iter().map(|node| node.plugins.len()).sum(),
        plugin_installation_count: backup
            .nodes
            .iter()
            .map(|node| node.plugin_installations.len())
            .sum(),
        workspace_setting_count: backup.workspace_settings.len(),
        remote_server_count: backup.remote_servers.len(),
        runtime_catalog_profile_count: backup.runtime_catalog_profiles.len(),
        runtime_signer_profile_count: backup.runtime_signer_profiles.len(),
        neo_wallet_profile_count: backup.neo_wallet_profiles.len(),
        fast_sync_snapshot_count: backup.fast_sync_snapshots.len(),
        event_count: backup.events.len(),
    }
}
