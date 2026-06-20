use std::collections::BTreeSet;

use anyhow::Result;

use super::uniqueness::insert_unique_value;
use crate::backup::{
    restore::{
        restored_fast_sync_snapshot, restored_neo_wallet_profile, restored_remote_server_profile,
        restored_runtime_catalog_profile, restored_runtime_signer_profile,
        restored_workspace_setting,
    },
    schema::WorkspaceBackup,
};

pub(super) fn validate_backup_profiles(backup: &WorkspaceBackup) -> Result<()> {
    validate_workspace_settings(backup)?;
    validate_remote_servers(backup)?;
    validate_runtime_catalog_profiles(backup)?;
    validate_runtime_signer_profiles(backup)?;
    validate_neo_wallet_profiles(backup)?;
    validate_fast_sync_snapshots(backup)
}

fn validate_workspace_settings(backup: &WorkspaceBackup) -> Result<()> {
    let mut setting_keys = BTreeSet::new();
    for setting in &backup.workspace_settings {
        insert_unique_value(
            &mut setting_keys,
            "workspace setting key",
            setting.key.trim(),
        )?;
        restored_workspace_setting(setting)?;
    }
    Ok(())
}

fn validate_remote_servers(backup: &WorkspaceBackup) -> Result<()> {
    let mut remote_server_ids = BTreeSet::new();
    let mut remote_server_urls = BTreeSet::new();
    for profile in &backup.remote_servers {
        insert_unique_value(
            &mut remote_server_ids,
            "remote server profile id",
            profile.id.trim(),
        )?;
        let restored = restored_remote_server_profile(profile)?;
        insert_unique_value(
            &mut remote_server_urls,
            "remote server base URL",
            restored.base_url.trim(),
        )?;
    }
    Ok(())
}

fn validate_runtime_catalog_profiles(backup: &WorkspaceBackup) -> Result<()> {
    let mut runtime_catalog_ids = BTreeSet::new();
    for profile in &backup.runtime_catalog_profiles {
        insert_unique_value(
            &mut runtime_catalog_ids,
            "runtime catalog profile id",
            profile.id.trim(),
        )?;
        restored_runtime_catalog_profile(profile)?;
    }
    Ok(())
}

fn validate_runtime_signer_profiles(backup: &WorkspaceBackup) -> Result<()> {
    let mut runtime_signer_ids = BTreeSet::new();
    for profile in &backup.runtime_signer_profiles {
        insert_unique_value(
            &mut runtime_signer_ids,
            "runtime signer profile id",
            profile.id.trim(),
        )?;
        restored_runtime_signer_profile(profile)?;
    }
    Ok(())
}

fn validate_neo_wallet_profiles(backup: &WorkspaceBackup) -> Result<()> {
    let mut neo_wallet_profile_ids = BTreeSet::new();
    for profile in &backup.neo_wallet_profiles {
        insert_unique_value(
            &mut neo_wallet_profile_ids,
            "Neo wallet profile id",
            profile.id.trim(),
        )?;
        restored_neo_wallet_profile(profile)?;
    }
    Ok(())
}

fn validate_fast_sync_snapshots(backup: &WorkspaceBackup) -> Result<()> {
    let mut snapshot_ids = BTreeSet::new();
    for snapshot in &backup.fast_sync_snapshots {
        insert_unique_value(
            &mut snapshot_ids,
            "Fast Sync snapshot id",
            snapshot.id.trim(),
        )?;
        restored_fast_sync_snapshot(snapshot)?;
    }
    Ok(())
}
