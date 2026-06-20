use anyhow::Result;

use crate::{
    federation::{validate_remote_server_profile, RemoteServerProfile},
    runtime::{
        validate_runtime_catalog_profile, validate_runtime_signer_profile, RuntimeCatalogProfile,
        RuntimeSignerProfile,
    },
    wallet::{validate_neo_wallet_profile, NeoWalletProfile},
};

use super::super::schema::{
    NeoWalletProfileBackup, RemoteServerProfileBackup, RuntimeCatalogProfileBackup,
    RuntimeSignerProfileBackup,
};

pub(in crate::backup) fn restored_remote_server_profile(
    backup: &RemoteServerProfileBackup,
) -> Result<RemoteServerProfile> {
    let profile = RemoteServerProfile {
        id: backup.id.clone(),
        name: backup.name.clone(),
        base_url: backup.base_url.clone(),
        description: backup.description.clone(),
        enabled: backup.enabled,
        created_at_unix: backup.created_at_unix,
        updated_at_unix: backup.updated_at_unix,
    };
    validate_remote_server_profile(&profile)?;
    Ok(profile)
}

pub(in crate::backup) fn restored_runtime_catalog_profile(
    backup: &RuntimeCatalogProfileBackup,
) -> Result<RuntimeCatalogProfile> {
    let profile = RuntimeCatalogProfile {
        id: backup.id.clone(),
        label: backup.label.clone(),
        source: backup.source.clone(),
        signature_source: backup.signature_source.clone(),
        ed25519_public_key: backup.ed25519_public_key.clone(),
        max_bytes: backup.max_bytes,
        enabled: backup.enabled,
        last_loaded_at_unix: backup.last_loaded_at_unix,
        last_signature_verified: backup.last_signature_verified,
        last_bytes: backup.last_bytes,
    };
    validate_runtime_catalog_profile(&profile)?;
    Ok(profile)
}

pub(in crate::backup) fn restored_runtime_signer_profile(
    backup: &RuntimeSignerProfileBackup,
) -> Result<RuntimeSignerProfile> {
    let profile = RuntimeSignerProfile {
        id: backup.id.clone(),
        label: backup.label.clone(),
        ed25519_public_key: backup.ed25519_public_key.clone(),
        enabled: backup.enabled,
        created_at_unix: backup.created_at_unix,
        last_used_at_unix: backup.last_used_at_unix,
    };
    validate_runtime_signer_profile(&profile)?;
    Ok(profile)
}

pub(in crate::backup) fn restored_neo_wallet_profile(
    backup: &NeoWalletProfileBackup,
) -> Result<NeoWalletProfile> {
    let profile = NeoWalletProfile {
        id: backup.id.clone(),
        label: backup.label.clone(),
        source_path: backup.source_path.clone(),
        wallet_version: backup.wallet_version.clone(),
        primary_address: backup.primary_address.clone(),
        contract_public_keys: backup.contract_public_keys.clone(),
        wallet_sha256: backup.wallet_sha256.clone(),
        account_count: backup.account_count,
        encrypted_account_count: backup.encrypted_account_count,
        default_account_count: backup.default_account_count,
        watch_only_account_count: backup.watch_only_account_count,
        validated_at_unix: backup.validated_at_unix,
        last_used_at_unix: backup.last_used_at_unix,
    };
    validate_neo_wallet_profile(&profile)?;
    Ok(profile)
}
