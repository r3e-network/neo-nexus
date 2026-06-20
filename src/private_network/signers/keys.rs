use super::super::*;

pub(in crate::private_network) fn normalize_public_key(value: &str) -> Result<String> {
    let key = value.trim().trim_start_matches("0x").to_ascii_lowercase();
    if key.len() != 66 {
        anyhow::bail!("committee public key must be a compressed 33-byte hex key");
    }
    if !key.starts_with("02") && !key.starts_with("03") {
        anyhow::bail!("committee public key must start with 02 or 03");
    }
    if !key.chars().all(|character| character.is_ascii_hexdigit()) {
        anyhow::bail!("committee public key must be hexadecimal");
    }
    Ok(key)
}

pub(in crate::private_network) fn has_signer_references(input: &str) -> bool {
    input.lines().any(|line| !line.trim().is_empty())
}
