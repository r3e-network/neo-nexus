use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

mod cache;
mod catalog;
mod catalog_io;
mod filter;
mod import;
mod io;
mod manager;
mod model;
mod validation;

pub use self::catalog::{FastSyncSnapshotCatalog, FastSyncSnapshotCatalogEntry};
pub use self::filter::{
    filter_snapshot_catalog_entries, filter_snapshots, SnapshotCatalogEntryFilter, SnapshotFilter,
};
pub use self::import::{SnapshotApplication, SnapshotImportMode};
pub use self::io::{sha256_bytes, sha256_file};
pub use self::manager::FastSyncSnapshotManager;
pub use self::model::{
    FastSyncSnapshot, NewFastSyncSnapshot, SnapshotCache, SnapshotCatalogLoad,
    SnapshotCatalogLoadRequest, SnapshotDownloadRequest, SnapshotVerification,
};
pub use self::validation::{
    normalize_sha256, validate_snapshot_catalog_entry, validate_snapshot_catalog_load_request,
    validate_snapshot_download_request, validate_snapshot_https_redirect, validate_snapshot_input,
};

pub(super) fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?
        .as_secs())
}
