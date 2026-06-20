use std::path::Path;

use super::model::FastSyncSnapshot;

pub(super) fn cache_filename(snapshot: &FastSyncSnapshot, sha256: &str) -> String {
    let file_name = snapshot
        .source_path
        .file_name()
        .and_then(|value| value.to_str())
        .or(snapshot.download_file_name.as_deref())
        .unwrap_or("snapshot.acc");
    cache_filename_parts(&snapshot.id, file_name, sha256)
}

pub(super) fn cache_filename_parts(snapshot_id: &str, file_name: &str, sha256: &str) -> String {
    let mut extension = snapshot_file_extension(file_name)
        .map(safe_fragment)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "snapshot".to_string());
    if extension == "download" {
        extension = "snapshot".to_string();
    }

    format!(
        "fastsync-{}-{}.{}",
        safe_fragment(snapshot_id),
        &sha256[..12],
        extension
    )
}

pub(super) fn snapshot_file_extension(file_name: &str) -> Option<&str> {
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz") {
        Some("tar.gz")
    } else if lower.ends_with(".tgz") {
        Some("tgz")
    } else {
        Path::new(file_name)
            .extension()
            .and_then(|value| value.to_str())
    }
}

pub(super) fn safe_fragment(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if matches!(character, '-' | '_' | '.') {
            output.push(character);
        } else if character.is_whitespace() {
            output.push('-');
        }
    }

    let trimmed = output.trim_matches(['-', '.', '_']).to_string();
    if trimmed.is_empty() {
        "snapshot".to_string()
    } else {
        trimmed
    }
}
