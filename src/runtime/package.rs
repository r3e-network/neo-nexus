mod download;
mod installation;
mod planning;
mod verification;

pub use planning::{RuntimeCatalogFleetPlan, RuntimeCatalogUpgradePlan, RuntimeUpgradePlan};

pub struct RuntimePackageManager;

impl RuntimePackageManager {
    pub const DEFAULT_DOWNLOAD_MAX_BYTES: u64 = 512 * 1024 * 1024;
    pub const DEFAULT_CATALOG_MAX_BYTES: u64 = 1024 * 1024;
}
