use anyhow::Result;

pub(in crate::release_pack) fn safe_file_name<'a>(value: &'a str, label: &str) -> Result<&'a str> {
    if value.trim().is_empty()
        || value.contains('/')
        || value.contains('\\')
        || value == "."
        || value == ".."
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
