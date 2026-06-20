use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::snapshots::{
    cache::safe_fragment, import::paths::SNAPSHOT_CONTROL_DIR, FastSyncSnapshot,
};

use super::super::paths::ensure_import_root;

pub(super) struct SnapshotImportDirs {
    pub(super) import_dir: PathBuf,
    pub(super) control_dir: PathBuf,
}

pub(super) fn prepare_snapshot_import_dirs(
    node_data_dir: &Path,
    snapshot: &FastSyncSnapshot,
) -> Result<SnapshotImportDirs> {
    let import_dir = runtime_import_dir(node_data_dir);
    ensure_import_root(&import_dir)?;

    let control_dir = node_data_dir
        .join(SNAPSHOT_CONTROL_DIR)
        .join(safe_fragment(&snapshot.id));
    fs::create_dir_all(&control_dir).with_context(|| {
        format!(
            "failed to create node fast sync directory {}",
            control_dir.display()
        )
    })?;

    Ok(SnapshotImportDirs {
        import_dir,
        control_dir,
    })
}

fn runtime_import_dir(node_data_dir: &Path) -> PathBuf {
    node_data_dir.to_path_buf()
}
