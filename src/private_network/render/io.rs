use super::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub(in crate::private_network) fn is_windows_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    (bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/'))
        || value.starts_with("\\\\")
}

pub(in crate::private_network) fn is_posix_absolute_path(value: &str) -> bool {
    value.starts_with('/') && !value.starts_with("//")
}

pub(in crate::private_network) fn write_script(
    path: &Path,
    text: &str,
    executable: bool,
) -> Result<()> {
    fs::write(path, text.as_bytes())
        .with_context(|| format!("failed to write private network script {}", path.display()))?;
    #[cfg(unix)]
    if executable {
        let mut permissions = fs::metadata(path)
            .with_context(|| {
                format!(
                    "failed to inspect private network script {}",
                    path.display()
                )
            })?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).with_context(|| {
            format!(
                "failed to mark private network script executable {}",
                path.display()
            )
        })?;
    }
    #[cfg(not(unix))]
    let _ = executable;
    Ok(())
}

pub(in crate::private_network) fn write_text_file(
    path: &Path,
    text: &str,
    label: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {label} directory {}", parent.display()))?;
    }
    fs::write(path, text.as_bytes())
        .with_context(|| format!("failed to write {label} {}", path.display()))?;
    Ok(())
}

pub(in crate::private_network) fn deployment_slug(
    template: PrivateNetworkTemplate,
    node_type: NodeType,
    network_magic: u32,
) -> String {
    format!(
        "{}-{}-{network_magic}",
        node_type,
        safe_fragment(template.label())
    )
}

pub(in crate::private_network) fn safe_fragment(value: &str) -> String {
    let fragment: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = fragment.trim_matches('-');
    if trimmed.is_empty() {
        "private-network".to_string()
    } else {
        trimmed.to_string()
    }
}

pub(in crate::private_network) fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_secs())
}
