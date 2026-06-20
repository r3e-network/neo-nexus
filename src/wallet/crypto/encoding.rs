pub(in crate::wallet) fn is_even_hex(value: &str) -> bool {
    !value.is_empty()
        && value.len().is_multiple_of(2)
        && value.chars().all(|character| character.is_ascii_hexdigit())
}

pub(super) fn hex_bytes(value: &str) -> Option<Vec<u8>> {
    if !is_even_hex(value) {
        return None;
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    for index in (0..value.len()).step_by(2) {
        bytes.push(u8::from_str_radix(&value[index..index + 2], 16).ok()?);
    }
    Some(bytes)
}
