use std::{
    env,
    path::{Path, PathBuf},
};

pub(super) fn find_signer_binary_on_path(binary: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    let candidate_names = signer_path_candidate_names(Path::new(binary));
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

#[cfg(windows)]
fn signer_path_candidate_names(path: &Path) -> Vec<PathBuf> {
    let pathext = env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
    signer_path_candidate_names_from_pathext(path, &pathext.to_string_lossy())
}

#[cfg(not(windows))]
fn signer_path_candidate_names(path: &Path) -> Vec<PathBuf> {
    vec![path.to_path_buf()]
}

#[cfg(any(windows, test))]
pub(super) fn signer_path_candidate_names_from_pathext(path: &Path, pathext: &str) -> Vec<PathBuf> {
    let raw = path.as_os_str().to_string_lossy();
    if path.extension().is_some() {
        return vec![PathBuf::from(raw.as_ref())];
    }

    let mut names = vec![PathBuf::from(raw.as_ref())];
    for extension in pathext.split(';').map(str::trim) {
        if !extension.is_empty() {
            names.push(PathBuf::from(format!("{raw}{extension}")));
        }
    }
    names
}
