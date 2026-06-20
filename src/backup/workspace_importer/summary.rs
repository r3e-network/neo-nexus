use anyhow::Result;

use super::super::{
    schema::{WorkspaceBackup, WorkspaceBackupValidation},
    validation::{validate_backup_collections, validate_backup_header},
};

pub(in crate::backup) fn validate_backup_summary(
    backup: &WorkspaceBackup,
) -> Result<WorkspaceBackupValidation> {
    validate_backup_header(backup)?;
    let counts = validate_backup_collections(backup)?;

    Ok(WorkspaceBackupValidation {
        source_path: None,
        schema_version: backup.schema_version,
        application_version: backup.application_version.clone(),
        exported_at_unix: backup.exported_at_unix,
        node_count: backup.nodes.len(),
        plugin_state_count: counts.plugin_state_count,
        plugin_installation_count: counts.plugin_installation_count,
        workspace_setting_count: backup.workspace_settings.len(),
        remote_server_count: backup.remote_servers.len(),
        runtime_catalog_profile_count: backup.runtime_catalog_profiles.len(),
        runtime_signer_profile_count: backup.runtime_signer_profiles.len(),
        neo_wallet_profile_count: backup.neo_wallet_profiles.len(),
        fast_sync_snapshot_count: backup.fast_sync_snapshots.len(),
        event_count: backup.events.len(),
    })
}
