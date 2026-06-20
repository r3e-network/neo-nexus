use std::{fs, path::Path};

use anyhow::{Context, Result};

use super::super::io::write_zip_archive;

pub(in crate::support_bundle) fn publish_support_bundle_archive(
    bundle_dir: &Path,
    archive_path: &Path,
) -> Result<()> {
    let temp_archive_path = archive_path.with_extension("zip.tmp");
    write_zip_archive(bundle_dir, &temp_archive_path)?;
    if archive_path.exists() {
        fs::remove_file(archive_path).with_context(|| {
            format!(
                "failed to replace existing support bundle archive {}",
                archive_path.display()
            )
        })?;
    }
    fs::rename(&temp_archive_path, archive_path).with_context(|| {
        format!(
            "failed to publish support bundle archive {}",
            archive_path.display()
        )
    })?;
    Ok(())
}
