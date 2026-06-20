use anyhow::Result;

use crate::{
    backup::{
        restore::{
            restored_fast_sync_snapshot, restored_neo_wallet_profile,
            restored_remote_server_profile, restored_runtime_catalog_profile,
            restored_runtime_signer_profile,
        },
        schema::WorkspaceBackup,
    },
    repository::Repository,
};

use super::counts::ProfileImportCounts;

pub(super) fn restore_profiles(
    repository: &Repository,
    backup: &WorkspaceBackup,
) -> Result<ProfileImportCounts> {
    let remote_servers = backup
        .remote_servers
        .iter()
        .map(restored_remote_server_profile)
        .collect::<Result<Vec<_>>>()?;
    for profile in &remote_servers {
        repository.upsert_remote_server_profile(profile)?;
    }

    let runtime_catalog_profiles = backup
        .runtime_catalog_profiles
        .iter()
        .map(restored_runtime_catalog_profile)
        .collect::<Result<Vec<_>>>()?;
    for profile in &runtime_catalog_profiles {
        repository.upsert_runtime_catalog_profile(profile)?;
    }

    let runtime_signer_profiles = backup
        .runtime_signer_profiles
        .iter()
        .map(restored_runtime_signer_profile)
        .collect::<Result<Vec<_>>>()?;
    for profile in &runtime_signer_profiles {
        repository.upsert_runtime_signer_profile(profile)?;
    }

    let neo_wallet_profiles = backup
        .neo_wallet_profiles
        .iter()
        .map(restored_neo_wallet_profile)
        .collect::<Result<Vec<_>>>()?;
    for profile in &neo_wallet_profiles {
        repository.upsert_neo_wallet_profile(profile)?;
    }

    let fast_sync_snapshots = backup
        .fast_sync_snapshots
        .iter()
        .map(restored_fast_sync_snapshot)
        .collect::<Result<Vec<_>>>()?;
    for snapshot in &fast_sync_snapshots {
        repository.upsert_fast_sync_snapshot(snapshot.clone())?;
    }

    Ok(ProfileImportCounts {
        remote_server_count: remote_servers.len(),
        runtime_catalog_profile_count: runtime_catalog_profiles.len(),
        runtime_signer_profile_count: runtime_signer_profiles.len(),
        neo_wallet_profile_count: neo_wallet_profiles.len(),
        fast_sync_snapshot_count: fast_sync_snapshots.len(),
    })
}
