use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::plugins::{fs_utils::ensure_real_directory_exists, PLUGIN_CONTROL_DIR};

pub(super) fn safe_archive_relative_path(path: &Path) -> Result<Option<PathBuf>> {
    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let text = part.to_string_lossy();
                if text.contains('\\')
                    || text.contains(':')
                    || text == "."
                    || text == ".."
                    || text.is_empty()
                {
                    anyhow::bail!(
                        "plugin package entry {} contains an unsafe path component",
                        path.display()
                    );
                }
                relative.push(part);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("plugin package entry {} is unsafe", path.display());
            }
        }
    }

    if relative.as_os_str().is_empty() {
        return Ok(None);
    }
    if archive_path_targets_control_dir(&relative) {
        anyhow::bail!(
            "plugin package entry {} targets NeoNexus control data",
            path.display()
        );
    }
    Ok(Some(relative))
}

pub(super) fn create_safe_directory(root: &Path, relative: &Path) -> Result<PathBuf> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            anyhow::bail!("plugin package directory {} is unsafe", relative.display());
        };
        current.push(part);
        ensure_real_directory_exists(&current, "plugin package directory")?;
    }
    Ok(current)
}

pub(super) fn prepare_new_archive_file(root: &Path, relative: &Path) -> Result<PathBuf> {
    if let Some(parent) = relative
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        create_safe_directory(root, parent)?;
    }
    let target = root.join(relative);
    match fs::symlink_metadata(&target) {
        Ok(_) => anyhow::bail!("plugin package target {} already exists", target.display()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(target),
        Err(error) => Err(error)
            .with_context(|| format!("failed to inspect plugin target {}", target.display())),
    }
}

fn archive_path_targets_control_dir(path: &Path) -> bool {
    path.components().next().is_some_and(|component| {
        matches!(component, Component::Normal(part) if part.to_string_lossy().eq_ignore_ascii_case(PLUGIN_CONTROL_DIR))
    })
}
