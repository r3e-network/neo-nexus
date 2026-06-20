use crate::{
    federation::RemoteServerProfile,
    runtime::{RuntimeCatalogProfile, RuntimeSignerProfile},
    wallet::NeoWalletProfile,
};

use super::super::schema::{
    NeoWalletProfileBackup, RemoteServerProfileBackup, RuntimeCatalogProfileBackup,
    RuntimeSignerProfileBackup,
};

pub(in crate::backup) fn remote_server_profile_backup(
    profile: RemoteServerProfile,
) -> RemoteServerProfileBackup {
    RemoteServerProfileBackup {
        id: profile.id,
        name: profile.name,
        base_url: profile.base_url,
        description: profile.description,
        enabled: profile.enabled,
        created_at_unix: profile.created_at_unix,
        updated_at_unix: profile.updated_at_unix,
    }
}

pub(in crate::backup) fn runtime_catalog_profile_backup(
    profile: RuntimeCatalogProfile,
) -> RuntimeCatalogProfileBackup {
    RuntimeCatalogProfileBackup {
        id: profile.id,
        label: profile.label,
        source: profile.source,
        signature_source: profile.signature_source,
        ed25519_public_key: profile.ed25519_public_key,
        max_bytes: profile.max_bytes,
        enabled: profile.enabled,
        last_loaded_at_unix: profile.last_loaded_at_unix,
        last_signature_verified: profile.last_signature_verified,
        last_bytes: profile.last_bytes,
    }
}

pub(in crate::backup) fn runtime_signer_profile_backup(
    profile: RuntimeSignerProfile,
) -> RuntimeSignerProfileBackup {
    RuntimeSignerProfileBackup {
        id: profile.id,
        label: profile.label,
        ed25519_public_key: profile.ed25519_public_key,
        enabled: profile.enabled,
        created_at_unix: profile.created_at_unix,
        last_used_at_unix: profile.last_used_at_unix,
    }
}

pub(in crate::backup) fn neo_wallet_profile_backup(
    profile: NeoWalletProfile,
) -> NeoWalletProfileBackup {
    NeoWalletProfileBackup {
        id: profile.id,
        label: profile.label,
        source_path: profile.source_path,
        wallet_version: profile.wallet_version,
        primary_address: profile.primary_address,
        contract_public_keys: profile.contract_public_keys,
        wallet_sha256: profile.wallet_sha256,
        account_count: profile.account_count,
        encrypted_account_count: profile.encrypted_account_count,
        default_account_count: profile.default_account_count,
        watch_only_account_count: profile.watch_only_account_count,
        validated_at_unix: profile.validated_at_unix,
        last_used_at_unix: profile.last_used_at_unix,
    }
}
