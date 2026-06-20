use anyhow::Result;

use super::ZipInstallResult;
use crate::plugins::{PLUGIN_PACKAGE_MAX_EXPANDED_BYTES, PLUGIN_PACKAGE_MAX_FILES};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ZipInstallTracker {
    installed_files: usize,
    expanded_bytes: u64,
}

impl ZipInstallTracker {
    pub(super) fn new() -> Self {
        Self {
            installed_files: 0,
            expanded_bytes: 0,
        }
    }

    pub(super) fn reserve_file(&mut self) -> Result<()> {
        if self.installed_files >= PLUGIN_PACKAGE_MAX_FILES {
            anyhow::bail!("plugin package contains too many files");
        }
        self.installed_files += 1;
        Ok(())
    }

    pub(super) fn reserve_bytes(&mut self, bytes: u64) -> Result<()> {
        let next = self.expanded_bytes.saturating_add(bytes);
        if next > PLUGIN_PACKAGE_MAX_EXPANDED_BYTES {
            anyhow::bail!(
                "plugin package expanded data exceeds {} bytes",
                PLUGIN_PACKAGE_MAX_EXPANDED_BYTES
            );
        }
        self.expanded_bytes = next;
        Ok(())
    }

    pub(super) fn finish(self) -> ZipInstallResult {
        ZipInstallResult {
            installed_files: self.installed_files,
            expanded_bytes: self.expanded_bytes,
        }
    }
}
