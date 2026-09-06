//! Bounded, permission-checked signer credential files.

use std::path::Path;

use anyhow::{bail, Context, Result};
use zeroize::Zeroizing;

use crate::secret_file::read_secret;

const MAX_TOKEN_FILE_BYTES: u64 = 4 * 1024;
const MAX_WORKLOAD_KEY_FILE_BYTES: u64 = 1024;
const MIN_BEARER_BYTES: usize = 32;

pub(super) fn read_token(path: &Path) -> Result<String> {
    let bytes = read_secret(path, MAX_TOKEN_FILE_BYTES, "signer admin token")?;
    let text = std::str::from_utf8(&bytes).with_context(|| {
        format!(
            "signer admin token file {} must contain valid UTF-8",
            path.display()
        )
    })?;
    let token = one_line(text).with_context(|| {
        format!(
            "signer admin token file {} must contain exactly one line",
            path.display()
        )
    })?;
    if token.len() < MIN_BEARER_BYTES {
        bail!(
            "signer admin token file {} must contain at least {MIN_BEARER_BYTES} bytes",
            path.display()
        );
    }
    if !valid_bearer_token(token) {
        bail!(
            "signer admin token file {} is not an RFC 6750 bearer token",
            path.display()
        );
    }
    Ok(token.to_string())
}

pub(super) fn read_workload_seed(path: &Path) -> Result<Zeroizing<[u8; 32]>> {
    let bytes = read_secret(path, MAX_WORKLOAD_KEY_FILE_BYTES, "signer workload key")?;
    let text = std::str::from_utf8(&bytes).with_context(|| {
        format!(
            "signer workload key file {} must contain valid UTF-8",
            path.display()
        )
    })?;
    let encoded = one_line(text).with_context(|| {
        format!(
            "signer workload key file {} must contain exactly one line",
            path.display()
        )
    })?;
    if encoded.len() != 64
        || !encoded
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        bail!(
            "signer workload key file {} must contain exactly 64 lowercase hexadecimal \
             characters (a 32-byte Ed25519 seed)",
            path.display()
        );
    }
    let mut seed = Zeroizing::new([0_u8; 32]);
    for (index, pair) in encoded.as_bytes().chunks_exact(2).enumerate() {
        let (Some(high), Some(low)) = (hex_nibble(pair[0]), hex_nibble(pair[1])) else {
            bail!("signer workload key failed validated hexadecimal decoding");
        };
        seed[index] = (high << 4) | low;
    }
    Ok(seed)
}

/// Accept no whitespace normalization except one conventional final line break.
fn one_line(text: &str) -> Result<&str> {
    let line = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(text);
    if line.is_empty() || line.contains(['\r', '\n']) || line.trim() != line {
        bail!("credential file is not a single unpadded line");
    }
    Ok(line)
}

fn valid_bearer_token(token: &str) -> bool {
    let core = token.trim_end_matches('=');
    !core.is_empty()
        && core.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'+' | b'/')
        })
        && token[core.len()..].bytes().all(|byte| byte == b'=')
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
