use anyhow::Result;

use crate::repository::Repository;

use super::super::{
    export::{
        event_backup, fast_sync_snapshot_backup, neo_wallet_profile_backup, node_backup,
        remote_server_profile_backup, runtime_catalog_profile_backup,
        runtime_signer_profile_backup, workspace_setting_backup,
    },
    schema::WorkspaceBackup,
};

pub(super) fn workspace_backup_snapshot(
    repository: &Repository,
    application_version: &str,
    exported_at_unix: u64,
) -> Result<WorkspaceBackup> {
    let nodes = repository
        .list_nodes()?
        .into_iter()
        .map(|node| node_backup(repository, node))
        .collect::<Result<Vec<_>>>()?;
    let events = repository
        .list_recent_events(256)?
        .into_iter()
        .map(event_backup)
        .collect();
    let workspace_settings = repository
        .list_workspace_settings_for_backup()?
        .into_iter()
        .map(workspace_setting_backup)
        .collect();
    let runtime_catalog_profiles = repository
        .list_runtime_catalog_profiles()?
        .into_iter()
        .map(runtime_catalog_profile_backup)
        .collect();
    let remote_servers = repository
        .list_remote_servers()?
        .into_iter()
        .map(remote_server_profile_backup)
        .collect();
    let runtime_signer_profiles = repository
        .list_runtime_signer_profiles()?
        .into_iter()
        .map(runtime_signer_profile_backup)
        .collect();
    let neo_wallet_profiles = repository
        .list_neo_wallet_profiles()?
        .into_iter()
        .map(neo_wallet_profile_backup)
        .collect();
    let fast_sync_snapshots = repository
        .list_fast_sync_snapshots()?
        .into_iter()
        .map(fast_sync_snapshot_backup)
        .collect();

    Ok(WorkspaceBackup {
        schema_version: 7,
        application: "NeoNexus".to_string(),
        application_version: application_version.to_string(),
        exported_at_unix,
        workspace_settings,
        remote_servers,
        runtime_catalog_profiles,
        runtime_signer_profiles,
        neo_wallet_profiles,
        fast_sync_snapshots,
        nodes,
        events,
    })
}
