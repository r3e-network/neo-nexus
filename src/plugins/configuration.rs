//! Package upgrades retain operator files and stage differences for review.
use crate::config::{prepare_plugin_config, stage_plugin_config};
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn configuration(path: &Path) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    if name.starts_with('.') && name.contains(".neonexus-") {
        return false;
    }
    // Executables and their debug metadata belong to the archive. All other
    // files can contain operator configuration or keys and must survive.
    !matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "dll" | "so" | "dylib" | "exe" | "pdb"
    )
}

fn files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        if fs::symlink_metadata(&directory)?.file_type().is_symlink() {
            anyhow::bail!("plugin config directory is a symbolic link");
        }
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            if entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(".neonexus")
            {
                continue;
            }
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                anyhow::bail!("plugin directory contains a symbolic link");
            }
            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() {
                result.push(entry.path().strip_prefix(root)?.to_path_buf());
            }
        }
    }
    Ok(result)
}

pub(super) fn preserve_configuration(target: &Path, staging: &Path, version: &str) -> Result<()> {
    let package_configs = files(staging)?
        .into_iter()
        .filter(|path| configuration(path))
        .collect::<Vec<_>>();
    let mut conflicts = Vec::new();
    for relative in &package_configs {
        let active = target.join(relative);
        let content = fs::read(staging.join(relative))?;
        if !prepare_plugin_config(&active, &content, version)? {
            conflicts.push(active.display().to_string());
        }
    }
    if !conflicts.is_empty() {
        anyhow::bail!("plugin configuration conflicts: {}. Existing plugin preserved; resolve candidates in /config and retry installation", conflicts.join(", "));
    }
    for relative in &package_configs {
        let staged = staging.join(relative);
        stage_plugin_config(
            &target.join(relative),
            &staged,
            &fs::read(&staged)?,
            version,
        )?;
    }
    for relative in files(target)? {
        let source = target.join(&relative);
        let destination = staging.join(&relative);
        let control = relative
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .contains(".neonexus-");
        if !destination.exists() && (configuration(&relative) || control) {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source, destination)?;
        }
    }
    Ok(())
}
