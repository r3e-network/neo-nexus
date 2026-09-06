use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use zip::ZipArchive;

mod copy;
mod paths;
mod tracker;

use self::{
    copy::copy_archive_file,
    paths::{create_safe_directory, prepare_new_archive_file, safe_archive_relative_path},
    tracker::ZipInstallTracker,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ZipInstallResult {
    pub(super) installed_files: usize,
    pub(super) expanded_bytes: u64,
}

pub(super) fn unpack_plugin_zip(
    source: &Path,
    target_dir: &Path,
    plugin_dir_name: &str,
) -> Result<ZipInstallResult> {
    let file = File::open(source)
        .with_context(|| format!("failed to open plugin package {}", source.display()))?;
    let mut archive = ZipArchive::new(file).context("failed to read plugin zip package")?;
    let official_prefix = official_layout_prefix(&mut archive, plugin_dir_name)?;
    let mut tracker = ZipInstallTracker::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .context("failed to read plugin zip package entry")?;
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            anyhow::bail!("plugin zip package contains a symbolic link entry");
        }
        let enclosed_name = entry
            .enclosed_name()
            .context("plugin zip package entry path is unsafe")?;
        let Some(mut relative_path) = safe_archive_relative_path(&enclosed_name)? else {
            continue;
        };
        if let Some(prefix) = &official_prefix {
            let Some(stripped) = strip_prefix_case_insensitive(&relative_path, prefix) else {
                anyhow::bail!(
                    "official plugin package entry {} escapes the expected {} layout",
                    relative_path.display(),
                    prefix.display()
                );
            };
            if stripped.as_os_str().is_empty() {
                continue;
            }
            relative_path = stripped;
        }

        if entry.is_dir() {
            create_safe_directory(target_dir, &relative_path)?;
        } else if entry.is_file() {
            let target_path = prepare_new_archive_file(target_dir, &relative_path)?;
            copy_archive_file(&mut entry, &target_path, &mut tracker)?;
        } else {
            anyhow::bail!(
                "plugin zip package entry {} has unsupported type",
                relative_path.display()
            );
        }
    }

    Ok(tracker.finish())
}

fn official_layout_prefix<R: std::io::Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    plugin_dir_name: &str,
) -> Result<Option<PathBuf>> {
    let expected = PathBuf::from("Plugins").join(plugin_dir_name);
    let mut paths = Vec::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .context("failed to inspect plugin zip package layout")?;
        let enclosed = entry
            .enclosed_name()
            .context("plugin zip package entry path is unsafe")?;
        if let Some(path) = safe_archive_relative_path(&enclosed)? {
            paths.push(path);
        }
    }
    if paths.is_empty() {
        return Ok(None);
    }
    let has_plugins_root = paths.iter().any(|path| {
        path.components().next().is_some_and(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case("Plugins")
        })
    });
    if !has_plugins_root {
        return Ok(None);
    }
    if paths
        .iter()
        .all(|path| strip_prefix_case_insensitive(path, &expected).is_some())
    {
        Ok(Some(expected))
    } else {
        anyhow::bail!(
            "plugin package contains a Plugins directory outside Plugins/{plugin_dir_name}"
        )
    }
}

fn strip_prefix_case_insensitive(path: &Path, prefix: &Path) -> Option<PathBuf> {
    let path_components = path.components().collect::<Vec<_>>();
    let prefix_components = prefix.components().collect::<Vec<_>>();
    if prefix_components.len() > path_components.len()
        || !prefix_components
            .iter()
            .zip(&path_components)
            .all(|(expected, actual)| {
                expected
                    .as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&actual.as_os_str().to_string_lossy())
            })
    {
        return None;
    }
    Some(path_components[prefix_components.len()..].iter().collect())
}
