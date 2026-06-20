use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

pub(in crate::release_pack) fn resolve_release_manifest(input: &Path) -> Result<PathBuf> {
    if input.is_dir() {
        return resolve_manifest_from_directory(input);
    }

    if is_release_manifest_path(input) {
        return Ok(input.to_path_buf());
    }

    if input.extension().and_then(|extension| extension.to_str()) == Some("zip") {
        return resolve_manifest_from_archive(input);
    }

    anyhow::bail!(
        "release package input must be a dist directory, .manifest.json, or .zip: {}",
        input.display()
    )
}

fn resolve_manifest_from_directory(input: &Path) -> Result<PathBuf> {
    let manifests = fs::read_dir(input)
        .with_context(|| format!("failed to inspect release directory {}", input.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to inspect release directory {}", input.display()))?
        .into_iter()
        .filter(|path| is_release_manifest_path(path))
        .collect::<Vec<_>>();

    match manifests.as_slice() {
        [manifest] => Ok(manifest.clone()),
        [] => anyhow::bail!("release directory {} has no manifest", input.display()),
        _ => anyhow::bail!(
            "release directory {} has multiple manifests; pass one manifest explicitly",
            input.display()
        ),
    }
}

fn resolve_manifest_from_archive(input: &Path) -> Result<PathBuf> {
    let archive_name = input
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .context("release archive path must be valid UTF-8")?;
    let manifest_name = archive_name.trim_end_matches(".zip").to_string() + ".manifest.json";
    let manifest_path = input.parent().map_or_else(
        || PathBuf::from(&manifest_name),
        |parent| parent.join(&manifest_name),
    );
    if manifest_path.is_file() {
        return Ok(manifest_path);
    }
    anyhow::bail!(
        "release archive {} has no sidecar manifest {}",
        input.display(),
        manifest_path.display()
    )
}

fn is_release_manifest_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|file_name| file_name.to_str())
        .is_some_and(|file_name| file_name.ends_with(".manifest.json"))
}
