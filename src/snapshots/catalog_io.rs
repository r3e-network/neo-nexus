mod http;
mod signature;
mod source;

pub(super) use http::fetch_https_response;
pub(super) use signature::{decode_fixed_base64, verify_detached_signature_bytes};
pub(super) use source::{
    is_https_source, optional_trimmed, read_catalog_source_bytes, SnapshotCatalogSource,
};

pub(super) const SNAPSHOT_CATALOG_SIGNATURE_MAX_BYTES: u64 = 64 * 1024;
