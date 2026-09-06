use anyhow::Result;

pub(in crate::release_pack) fn safe_file_name<'a>(value: &'a str, label: &str) -> Result<&'a str> {
    const MAX_FILE_NAME_BYTES: usize = 128;
    if value.len() > MAX_FILE_NAME_BYTES {
        anyhow::bail!("release {label} exceeds {MAX_FILE_NAME_BYTES} bytes");
    }
    if value.trim().is_empty()
        || !value.is_ascii()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        || value.starts_with('.')
        || value.ends_with('.')
    {
        anyhow::bail!("release {label} must be a simple file name: {value}");
    }
    Ok(value)
}

pub(in crate::release_pack) fn safe_fragment(value: &str) -> String {
    let fragment = value
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if fragment.is_empty() {
        "unknown".to_string()
    } else {
        fragment
    }
}

pub(in crate::release_pack) fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit()) {
        Ok(())
    } else {
        anyhow::bail!("{label} must be a 64-character hex digest")
    }
}
