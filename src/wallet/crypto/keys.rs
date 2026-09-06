use super::{
    base58::{base58check_payload, is_base58_char},
    encoding::hex_bytes,
    hash::hash160,
};

pub(crate) fn valid_nep2_key(value: &str) -> bool {
    let trimmed = value.trim();
    if !trimmed.starts_with("6P") || trimmed.len() != 58 {
        return false;
    }
    match base58check_payload(trimmed) {
        Some(payload) => payload.len() == 39 && payload.starts_with(&[0x01, 0x42, 0xe0]),
        None => false,
    }
}

pub(crate) fn neo_address_payload(value: &str) -> Option<Vec<u8>> {
    let trimmed = value.trim();
    if trimmed.len() != 34 || !trimmed.is_ascii() {
        return None;
    }
    base58check_payload(trimmed)
        .filter(|payload| payload.len() == 21 && payload.first() == Some(&0x35))
}

/// Neo N3 single-signature verification script:
/// `0c21 <33B compressed pubkey> 4156e7b327` — `PUSHDATA1 33`, the key, then
/// the `System.Contract.CreateStandardAccount` syscall. The N2 shape
/// (`21 <pubkey> ac`) is not accepted: every real N3 wallet carries the
/// syscall form.
pub(crate) fn extract_single_sig_contract_public_key(script: &str) -> Option<String> {
    let normalized = script.trim().trim_start_matches("0x").to_ascii_lowercase();
    if normalized.len() != 80
        || !normalized.starts_with("0c21")
        || !normalized.ends_with("4156e7b327")
    {
        return None;
    }
    let public_key = &normalized[4..70];
    valid_compressed_public_key(public_key).then(|| public_key.to_string())
}

pub(crate) fn valid_compressed_public_key(value: &str) -> bool {
    value.len() == 66
        && (value.starts_with("02") || value.starts_with("03"))
        && value.chars().all(|character| character.is_ascii_hexdigit())
        && hex_bytes(value)
            .as_deref()
            .is_some_and(|bytes| p256::PublicKey::from_sec1_bytes(bytes).is_ok())
}

pub(crate) fn script_hash_from_hex(script: &str) -> Option<Vec<u8>> {
    let script_bytes = hex_bytes(script)?;
    Some(hash160(&script_bytes))
}

pub(crate) fn looks_like_plain_private_key(value: &str) -> bool {
    let trimmed = value.trim();
    (trimmed.len() == 52
        && matches!(trimmed.as_bytes().first(), Some(b'K' | b'L' | b'5'))
        && trimmed.chars().all(is_base58_char))
        || (trimmed.len() == 64
            && trimmed
                .chars()
                .all(|character| character.is_ascii_hexdigit()))
}
