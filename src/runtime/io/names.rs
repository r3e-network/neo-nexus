use std::path::Path;

use anyhow::Result;

pub(in crate::runtime) fn safe_file_name(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("runtime executable name is required");
    }
    if trimmed == "." || trimmed == ".." || trimmed.contains('/') || trimmed.contains('\\') {
        anyhow::bail!("runtime executable name must not contain path separators");
    }
    Ok(trimmed.to_string())
}

pub(in crate::runtime) fn cache_file_name(file_name: &str, sha256: &str) -> String {
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .map(safe_fragment)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime".to_string());
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(safe_fragment)
        .filter(|value| !value.is_empty());

    match extension {
        Some(extension) => format!("{stem}-{}.{}", &sha256[..12], extension),
        None => format!("{stem}-{}", &sha256[..12]),
    }
}

pub(in crate::runtime) fn safe_fragment(value: &str) -> String {
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
        "runtime".to_string()
    } else {
        trimmed
    }
}
