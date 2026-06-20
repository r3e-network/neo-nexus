use super::super::*;

pub(in crate::private_network) fn safe_launch_pack_child(
    root_path: &Path,
    value: &str,
) -> Option<PathBuf> {
    let path = Path::new(value);
    if value.trim().is_empty()
        || path.is_absolute()
        || is_windows_path(value)
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return None;
    }
    Some(root_path.join(path))
}

pub(in crate::private_network) fn resolve_launch_pack_reference(
    root_path: &Path,
    value: &str,
) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() || is_windows_path(value) {
        path.to_path_buf()
    } else {
        root_path.join(path)
    }
}

pub(super) fn signer_wallet_path_is_foreign(value: &str) -> bool {
    if cfg!(windows) {
        is_posix_absolute_path(value)
    } else {
        is_windows_path(value)
    }
}

pub(super) fn signer_sidecar_binary_is_foreign(value: &str) -> bool {
    if cfg!(windows) {
        is_posix_absolute_path(value)
    } else {
        is_windows_path(value) || value.contains('\\')
    }
}

pub(super) fn signer_binary_should_search_path(path: &Path, raw: &str) -> bool {
    !raw.contains('/')
        && !raw.contains('\\')
        && !path.is_absolute()
        && !is_windows_path(raw)
        && path.components().count() == 1
}
