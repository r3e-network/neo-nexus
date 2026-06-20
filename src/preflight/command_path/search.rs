use std::path::Path;

pub(in crate::preflight) fn should_search_path(path: &Path) -> bool {
    !path.is_absolute() && path.components().count() == 1
}

pub(super) fn path_candidate_names(path: &Path) -> Vec<std::path::PathBuf> {
    platform::path_candidate_names(path)
}

#[cfg(windows)]
mod platform {
    use std::{
        env,
        path::{Path, PathBuf},
    };

    pub(super) fn path_candidate_names(path: &Path) -> Vec<PathBuf> {
        let raw = path.as_os_str().to_string_lossy();
        if path.extension().is_some() {
            return vec![PathBuf::from(raw.as_ref())];
        }

        let mut names = vec![PathBuf::from(raw.as_ref())];
        let pathext = env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
        for extension in pathext.to_string_lossy().split(';') {
            let extension = extension.trim();
            if !extension.is_empty() {
                names.push(PathBuf::from(format!("{raw}{extension}")));
            }
        }
        names
    }
}

#[cfg(not(windows))]
mod platform {
    use std::path::{Path, PathBuf};

    pub(super) fn path_candidate_names(path: &Path) -> Vec<PathBuf> {
        vec![path.to_path_buf()]
    }
}
