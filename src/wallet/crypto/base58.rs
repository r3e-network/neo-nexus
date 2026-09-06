use super::hash::double_sha256;

const BASE58_ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

pub(crate) fn base58check_payload(value: &str) -> Option<Vec<u8>> {
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

pub(crate) fn base58check_encode(payload: &[u8]) -> String {
    let digest = double_sha256(payload);
    let mut bytes = Vec::with_capacity(payload.len() + 4);
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&digest[..4]);
    base58_encode(&bytes)
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

fn base58_encode(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let mut digits = Vec::<u8>::new();
    for byte in bytes {
        let mut carry = u32::from(*byte);
        for digit in digits.iter_mut().rev() {
            let value = u32::from(*digit) * 256 + carry;
            *digit = (value % 58) as u8;
            carry = value / 58;
        }
        while carry > 0 {
            digits.insert(0, (carry % 58) as u8);
            carry /= 58;
        }
    }
    let mut encoded = bytes
        .iter()
        .take_while(|byte| **byte == 0)
        .map(|_| '1')
        .collect::<String>();
    for digit in digits {
        let index = usize::from(digit);
        if let Some(character) = BASE58_ALPHABET.as_bytes().get(index) {
            encoded.push(char::from(*character));
        }
    }
    encoded
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
