use super::{
    base58::{base58check_payload, is_base58_char},
    encoding::hex_bytes,
    hash::hash160,
};

pub(in crate::wallet) fn valid_nep2_key(value: &str) -> bool {
    let trimmed = value.trim();
    if !trimmed.starts_with("6P") || trimmed.len() != 58 {
        return false;
    }
    match base58check_payload(trimmed) {
        Some(payload) => payload.len() == 39 && payload.starts_with(&[0x01, 0x42]),
        None => false,
    }
}

pub(in crate::wallet) fn neo_address_payload(value: &str) -> Option<Vec<u8>> {
    let trimmed = value.trim();
    base58check_payload(trimmed).filter(|payload| payload.len() == 21)
}

pub(in crate::wallet) fn extract_single_sig_contract_public_key(script: &str) -> Option<String> {
    let normalized = script.trim().trim_start_matches("0x").to_ascii_lowercase();
    if normalized.len() != 70 || !normalized.starts_with("21") || !normalized.ends_with("ac") {
        return None;
    }
    let public_key = &normalized[2..68];
    valid_compressed_public_key(public_key).then(|| public_key.to_string())
}

pub(in crate::wallet) fn valid_compressed_public_key(value: &str) -> bool {
    value.len() == 66
        && (value.starts_with("02") || value.starts_with("03"))
        && value.chars().all(|character| character.is_ascii_hexdigit())
}

pub(in crate::wallet) fn script_hash_from_hex(script: &str) -> Option<Vec<u8>> {
    let script_bytes = hex_bytes(script)?;
    Some(hash160(&script_bytes))
}

pub(in crate::wallet) fn looks_like_plain_private_key(value: &str) -> bool {
    let trimmed = value.trim();
    (trimmed.len() == 52
        && matches!(trimmed.as_bytes().first(), Some(b'K' | b'L' | b'5'))
        && trimmed.chars().all(is_base58_char))
        || (trimmed.len() == 64
            && trimmed
                .chars()
                .all(|character| character.is_ascii_hexdigit()))
}
