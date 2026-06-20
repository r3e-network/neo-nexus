mod application;
mod cache_ops;
mod catalog_ops;
mod verification;

pub struct FastSyncSnapshotManager;

impl FastSyncSnapshotManager {
    pub const DEFAULT_CATALOG_MAX_BYTES: u64 = 1024 * 1024;
    pub const DEFAULT_DOWNLOAD_MAX_BYTES: u64 = 64 * 1024 * 1024 * 1024;
}
