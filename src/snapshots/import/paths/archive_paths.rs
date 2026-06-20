use std::path::{Component, Path, PathBuf};

use anyhow::Result;

use super::root::SNAPSHOT_CONTROL_DIR;

pub(in crate::snapshots::import) fn safe_archive_relative_path(
    path: &Path,
) -> Result<Option<PathBuf>> {
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
                        "snapshot archive entry {} contains an unsafe path component",
                        path.display()
                    );
                }
                relative.push(part);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("snapshot archive entry {} is unsafe", path.display());
            }
        }
    }

    if relative.as_os_str().is_empty() {
        return Ok(None);
    }
    if archive_path_targets_control_dir(&relative) {
        anyhow::bail!(
            "snapshot archive entry {} targets NeoNexus control data",
            path.display()
        );
    }
    Ok(Some(relative))
}

fn archive_path_targets_control_dir(path: &Path) -> bool {
    path.components().next().is_some_and(|component| {
        matches!(component, Component::Normal(part) if {
            let text = part.to_string_lossy();
            text.eq_ignore_ascii_case(SNAPSHOT_CONTROL_DIR)
                || text.eq_ignore_ascii_case(".neonexus")
        })
    })
}
