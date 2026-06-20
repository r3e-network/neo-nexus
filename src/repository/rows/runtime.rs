use std::path::PathBuf;

use crate::{
    runtime::{RuntimeCatalogProfile, RuntimeInstallation, RuntimePlatform, RuntimeSignerProfile},
    types::NodeType,
};

use super::parse_field;

pub(in crate::repository) fn runtime_installation_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<RuntimeInstallation> {
    let node_type_raw: String = row.get(2)?;
    Ok(RuntimeInstallation {
        package_id: row.get(0)?,
        label: row.get(1)?,
        node_type: parse_field::<NodeType>(&node_type_raw)?,
        version: row.get(3)?,
        platform: RuntimePlatform {
            os: row.get(4)?,
            arch: row.get(5)?,
        },
        binary_path: PathBuf::from(row.get::<_, String>(6)?),
        sha256: row.get(7)?,
        signature_verified: row.get(8)?,
        signer_public_key: row.get(9)?,
        bytes: row.get(10)?,
        installed_at_unix: row.get(11)?,
    })
}

pub(in crate::repository) fn runtime_catalog_profile_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<RuntimeCatalogProfile> {
    Ok(RuntimeCatalogProfile {
        id: row.get(0)?,
        label: row.get(1)?,
        source: row.get(2)?,
        signature_source: row.get(3)?,
        ed25519_public_key: row.get(4)?,
        max_bytes: row.get(5)?,
        enabled: row.get(6)?,
        last_loaded_at_unix: row.get(7)?,
        last_signature_verified: row.get(8)?,
        last_bytes: row.get(9)?,
    })
}

pub(in crate::repository) fn runtime_signer_profile_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<RuntimeSignerProfile> {
    Ok(RuntimeSignerProfile {
        id: row.get(0)?,
        label: row.get(1)?,
        ed25519_public_key: row.get(2)?,
        enabled: row.get(3)?,
        created_at_unix: row.get(4)?,
        last_used_at_unix: row.get(5)?,
    })
}
