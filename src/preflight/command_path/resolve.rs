use std::{
    env,
    path::{Path, PathBuf},
};

use super::search::{path_candidate_names, should_search_path};

pub fn resolve_command_path(binary_path: &Path) -> Option<PathBuf> {
    if binary_path.as_os_str().is_empty() {
        return None;
    }

    if binary_path.is_file() {
        return Some(binary_path.to_path_buf());
    }

    if !should_search_path(binary_path) {
        return None;
    }

    resolve_from_path(binary_path)
}

fn resolve_from_path(binary_path: &Path) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    let candidate_names = path_candidate_names(binary_path);
    for directory in env::split_paths(&paths) {
        for candidate_name in &candidate_names {
            let candidate = directory.join(candidate_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}
