use anyhow::{Context, Result};

use super::super::{current_unix_time, verify_detached_signature_bytes};
use super::{
    io::{read_catalog_source_bytes, RuntimeCatalogSource},
    validation::{optional_trimmed, validate_catalog_load_request},
    RuntimeCatalogLoad, RuntimeCatalogLoadRequest, RuntimeReleaseCatalog,
};

const CATALOG_SIGNATURE_MAX_BYTES: u64 = 64 * 1024;

pub(in crate::runtime) fn load_release_catalog(
    request: &RuntimeCatalogLoadRequest,
) -> Result<RuntimeCatalogLoad> {
    validate_catalog_load_request(request)?;
    let source = request.source.trim();
    let source_location = RuntimeCatalogSource::parse(source, "runtime catalog source")?;
    let catalog_bytes =
        read_catalog_source_bytes(source_location, request.max_bytes, "runtime catalog")?;

    let signature_verified = if let (Some(signature_source), Some(public_key)) = (
        optional_trimmed(&request.signature_source),
        optional_trimmed(&request.ed25519_public_key),
    ) {
        let signature_location =
            RuntimeCatalogSource::parse(signature_source, "runtime catalog signature")?;
        let signature_bytes = read_catalog_source_bytes(
            signature_location,
            CATALOG_SIGNATURE_MAX_BYTES,
            "runtime catalog signature",
        )?;
        let verified =
            verify_detached_signature_bytes(&catalog_bytes, &signature_bytes, public_key)?;
        if !verified {
            anyhow::bail!("runtime catalog signature verification failed");
        }
        Some(true)
    } else {
        None
    };

    let text = std::str::from_utf8(&catalog_bytes)
        .context("runtime release catalog must be UTF-8 JSON")?;
    let catalog = RuntimeReleaseCatalog::from_json(text)?;

    Ok(RuntimeCatalogLoad {
        catalog,
        source: source.to_string(),
        bytes: catalog_bytes.len() as u64,
        signature_verified,
        loaded_at_unix: current_unix_time()?,
    })
}
