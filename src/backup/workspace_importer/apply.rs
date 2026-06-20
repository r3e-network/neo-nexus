use anyhow::Result;

use crate::repository::Repository;

mod counts;
mod events;
mod nodes;
mod profiles;
mod settings;

use super::super::schema::{WorkspaceBackup, WorkspaceBackupImport};
use super::summary::validate_backup_summary;

use events::restore_events;
use nodes::restore_nodes;
use profiles::restore_profiles;
use settings::restore_workspace_settings;

pub(in crate::backup) fn import_backup(
    repository: &Repository,
    backup: &WorkspaceBackup,
) -> Result<WorkspaceBackupImport> {
    let validation = validate_backup_summary(backup)?;
    let workspace_setting_count = restore_workspace_settings(repository, backup)?;
    let profile_counts = restore_profiles(repository, backup)?;
    let node_counts = restore_nodes(repository, backup)?;
    let event_count = restore_events(repository, backup)?;

    Ok(WorkspaceBackupImport {
        source_path: None,
        created_nodes: node_counts.created_nodes,
        updated_nodes: node_counts.updated_nodes,
        plugin_state_count: node_counts.plugin_state_count,
        plugin_installation_count: node_counts.plugin_installation_count,
        workspace_setting_count,
        remote_server_count: profile_counts.remote_server_count,
        runtime_catalog_profile_count: profile_counts.runtime_catalog_profile_count,
        runtime_signer_profile_count: profile_counts.runtime_signer_profile_count,
        neo_wallet_profile_count: profile_counts.neo_wallet_profile_count,
        fast_sync_snapshot_count: profile_counts.fast_sync_snapshot_count,
        event_count,
        schema_version: validation.schema_version,
        exported_at_unix: validation.exported_at_unix,
    })
}
