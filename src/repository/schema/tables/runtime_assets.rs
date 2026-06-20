use anyhow::Result;
use rusqlite::Connection;

pub(super) fn create_runtime_asset_tables(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS fast_sync_snapshots (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            network TEXT NOT NULL,
            node_type TEXT NOT NULL,
            source_path TEXT NOT NULL,
            source_url TEXT,
            download_file_name TEXT,
            download_max_bytes INTEGER NOT NULL DEFAULT 68719476736,
            expected_sha256 TEXT NOT NULL,
            cached_path TEXT,
            verified_sha256 TEXT,
            verified_at_unix INTEGER,
            bytes INTEGER
        );
        CREATE TABLE IF NOT EXISTS runtime_installations (
            package_id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            node_type TEXT NOT NULL,
            version TEXT NOT NULL,
            os TEXT NOT NULL,
            arch TEXT NOT NULL,
            binary_path TEXT NOT NULL,
            sha256 TEXT NOT NULL,
            signature_verified INTEGER NOT NULL DEFAULT 0,
            signer_public_key TEXT,
            bytes INTEGER NOT NULL,
            installed_at_unix INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS runtime_catalog_profiles (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            source TEXT NOT NULL,
            signature_source TEXT,
            ed25519_public_key TEXT,
            max_bytes INTEGER NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            last_loaded_at_unix INTEGER,
            last_signature_verified INTEGER,
            last_bytes INTEGER
        );
        CREATE TABLE IF NOT EXISTS runtime_signer_profiles (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            ed25519_public_key TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at_unix INTEGER NOT NULL,
            last_used_at_unix INTEGER
        );
        CREATE TABLE IF NOT EXISTS neo_wallet_profiles (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            source_path TEXT NOT NULL,
            wallet_version TEXT,
            primary_address TEXT NOT NULL,
            contract_public_keys TEXT NOT NULL,
            wallet_sha256 TEXT NOT NULL,
            account_count INTEGER NOT NULL,
            encrypted_account_count INTEGER NOT NULL,
            default_account_count INTEGER NOT NULL,
            watch_only_account_count INTEGER NOT NULL,
            validated_at_unix INTEGER NOT NULL,
            last_used_at_unix INTEGER
        );",
    )?;
    Ok(())
}
