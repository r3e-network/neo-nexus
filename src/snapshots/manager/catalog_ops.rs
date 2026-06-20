use anyhow::{Context, Result};

use super::FastSyncSnapshotManager;
use crate::snapshots::{
    catalog::FastSyncSnapshotCatalog,
    catalog_io::{
        optional_trimmed, read_catalog_source_bytes, verify_detached_signature_bytes,
        SnapshotCatalogSource, SNAPSHOT_CATALOG_SIGNATURE_MAX_BYTES,
    },
    current_unix_time,
    model::{SnapshotCatalogLoad, SnapshotCatalogLoadRequest},
    validation::validate_snapshot_catalog_load_request,
};

impl FastSyncSnapshotManager {
    pub fn load_catalog(request: &SnapshotCatalogLoadRequest) -> Result<SnapshotCatalogLoad> {
        validate_snapshot_catalog_load_request(request)?;
        let source = request.source.trim();
        let source_location = SnapshotCatalogSource::parse(source, "snapshot catalog source")?;
        let catalog_bytes =
            read_catalog_source_bytes(source_location, request.max_bytes, "snapshot catalog")?;

        let signature_verified = if let (Some(signature_source), Some(public_key)) = (
            optional_trimmed(&request.signature_source),
            optional_trimmed(&request.ed25519_public_key),
        ) {
            let signature_location =
                SnapshotCatalogSource::parse(signature_source, "snapshot catalog signature")?;
            let signature_bytes = read_catalog_source_bytes(
                signature_location,
                SNAPSHOT_CATALOG_SIGNATURE_MAX_BYTES,
                "snapshot catalog signature",
            )?;
            let verified =
                verify_detached_signature_bytes(&catalog_bytes, &signature_bytes, public_key)?;
            if !verified {
                anyhow::bail!("snapshot catalog signature verification failed");
            }
            Some(true)
        } else {
            None
        };

        let text =
            std::str::from_utf8(&catalog_bytes).context("snapshot catalog must be UTF-8 JSON")?;
        let catalog = FastSyncSnapshotCatalog::from_json(text)?;

        Ok(SnapshotCatalogLoad {
            catalog,
            source: source.to_string(),
            bytes: catalog_bytes.len() as u64,
            signature_verified,
            loaded_at_unix: current_unix_time()?,
        })
    }
}
