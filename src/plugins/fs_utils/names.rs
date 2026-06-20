use std::path::{Path, PathBuf};

fn safe_fragment(value: &str) -> String {
    let fragment: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = fragment.trim_matches('-');
    if trimmed.is_empty() {
        "plugin".to_string()
    } else {
        trimmed.to_string()
    }
}

pub(in crate::plugins) fn staging_dir(
    control_root: &Path,
    plugin_dir_name: &str,
    installed_at_unix: u64,
) -> PathBuf {
    control_root.join("staging").join(format!(
        "{}-{}",
        safe_fragment(plugin_dir_name),
        installed_at_unix
    ))
}

pub(in crate::plugins) fn backup_dir(
    control_root: &Path,
    plugin_dir_name: &str,
    installed_at_unix: u64,
) -> PathBuf {
    control_root.join("replace-backups").join(format!(
        "{}-{}",
        safe_fragment(plugin_dir_name),
        installed_at_unix
    ))
}
