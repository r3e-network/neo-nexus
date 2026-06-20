use anyhow::Result;

pub fn normalize_sha256(value: &str) -> Result<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.len() != 64
        || !normalized
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        anyhow::bail!("SHA-256 must be 64 hexadecimal characters");
    }
    Ok(normalized)
}
