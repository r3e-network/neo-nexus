use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::snapshots::sha256_file;

use super::SupportBundleFile;

pub(super) fn write_bundle_file(
    root: &Path,
    relative_path: &str,
    content: &str,
    files: &mut Vec<SupportBundleFile>,
) -> Result<()> {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create support bundle directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(&path, content)
        .with_context(|| format!("failed to write support bundle file {}", path.display()))?;
    files.push(file_evidence(root, &path)?);
    Ok(())
}

fn file_evidence(root: &Path, path: &Path) -> Result<SupportBundleFile> {
    let (sha256, bytes) = sha256_file(path)?;
    let relative = path.strip_prefix(root).with_context(|| {
        format!(
            "support bundle file {} is outside {}",
            path.display(),
            root.display()
        )
    })?;
    Ok(SupportBundleFile {
        path: zip_entry_name(relative),
        bytes,
        sha256,
    })
}

pub(super) fn write_zip_archive(root: &Path, archive_path: &Path) -> Result<()> {
    let file = File::create(archive_path).with_context(|| {
        format!(
            "failed to create support bundle archive {}",
            archive_path.display()
        )
    })?;
    let mut zip = ZipWriter::new(file);
    add_zip_entries(root, root, &mut zip)?;
    zip.finish()
        .context("failed to finish support bundle archive")?;
    Ok(())
}

fn add_zip_entries(root: &Path, directory: &Path, zip: &mut ZipWriter<File>) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| {
            format!(
                "failed to read support bundle directory {}",
                directory.display()
            )
        })?
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| {
            format!(
                "failed to read support bundle directory {}",
                directory.display()
            )
        })?;
    entries.sort_by_key(|entry| entry.path());
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            add_zip_entries(root, &path, zip)?;
        } else if path.is_file() {
            let relative = path.strip_prefix(root).with_context(|| {
                format!(
                    "support bundle file {} is outside {}",
                    path.display(),
                    root.display()
                )
            })?;
            zip.start_file(zip_entry_name(relative), options)
                .with_context(|| {
                    format!("failed to add {} to support bundle archive", path.display())
                })?;
            let mut input = File::open(&path).with_context(|| {
                format!("failed to read support bundle file {}", path.display())
            })?;
            let mut buffer = Vec::new();
            input.read_to_end(&mut buffer).with_context(|| {
                format!("failed to read support bundle file {}", path.display())
            })?;
            zip.write_all(&buffer).with_context(|| {
                format!(
                    "failed to write {} into support bundle archive",
                    path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn zip_entry_name(path: &Path) -> String {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}
