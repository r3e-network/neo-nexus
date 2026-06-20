use super::hash::double_sha256;

const BASE58_ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

pub(super) fn base58check_payload(value: &str) -> Option<Vec<u8>> {
    let decoded = base58_decode(value)?;
    if decoded.len() < 5 {
        return None;
    }
    let (payload, checksum) = decoded.split_at(decoded.len() - 4);
    let digest = double_sha256(payload);
    if checksum == &digest[..4] {
        Some(payload.to_vec())
    } else {
        None
    }
}

fn base58_decode(value: &str) -> Option<Vec<u8>> {
    if value.is_empty() {
        return None;
    }

    let mut bytes = Vec::<u8>::new();
    for character in value.chars() {
        let mut carry = base58_value(character)?;
        for byte in bytes.iter_mut().rev() {
            let value = (*byte as u32) * 58 + carry;
            *byte = (value & 0xff) as u8;
            carry = value >> 8;
        }
        while carry > 0 {
            bytes.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }

    let leading_zeroes = value
        .chars()
        .take_while(|character| *character == '1')
        .count();
    for _ in 0..leading_zeroes {
        bytes.insert(0, 0);
    }

    Some(bytes)
}

fn base58_value(character: char) -> Option<u32> {
    BASE58_ALPHABET
        .chars()
        .position(|candidate| candidate == character)
        .map(|index| index as u32)
}

pub(super) fn is_base58_char(character: char) -> bool {
    base58_value(character).is_some()
}
