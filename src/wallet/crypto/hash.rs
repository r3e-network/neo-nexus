use std::fmt::Write as _;

use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

pub(crate) fn double_sha256(bytes: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(bytes);
    let second = Sha256::digest(first);
    second.into()
}

pub(crate) fn hash160(bytes: &[u8]) -> Vec<u8> {
    let sha256 = Sha256::digest(bytes);
    Ripemd160::digest(sha256).to_vec()
}
