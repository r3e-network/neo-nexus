use std::path::Path;

use anyhow::Result;

use super::SnapshotImport;

const SNAPSHOT_IMPORT_MAX_EXPANDED_BYTES: u64 = 256 * 1024 * 1024 * 1024;
const SNAPSHOT_IMPORT_MAX_FILES: usize = 100_000;

pub(super) struct ArchiveImportTracker {
    imported_files: usize,
    expanded_bytes: u64,
}

impl ArchiveImportTracker {
    pub(super) fn new() -> Self {
        Self {
            imported_files: 0,
            expanded_bytes: 0,
        }
    }

    pub(super) fn reserve_file(&mut self) -> Result<()> {
        if self.imported_files >= SNAPSHOT_IMPORT_MAX_FILES {
            anyhow::bail!("snapshot archive contains too many files");
        }
        self.imported_files += 1;
        Ok(())
    }

    pub(super) fn reserve_bytes(&mut self, bytes: u64) -> Result<()> {
        let next = self.expanded_bytes.saturating_add(bytes);
        if next > SNAPSHOT_IMPORT_MAX_EXPANDED_BYTES {
            anyhow::bail!(
                "snapshot archive expanded data exceeds {} bytes",
                SNAPSHOT_IMPORT_MAX_EXPANDED_BYTES
            );
        }
        self.expanded_bytes = next;
        Ok(())
    }

    pub(super) fn finish(self, target_dir: &Path) -> SnapshotImport {
        SnapshotImport {
            snapshot_path: target_dir.to_path_buf(),
            imported_files: self.imported_files,
            expanded_bytes: self.expanded_bytes,
        }
    }
}
