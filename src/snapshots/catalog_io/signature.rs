use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

pub(in crate::snapshots) fn verify_detached_signature_bytes(
    payload: &[u8],
    signature: &[u8],
    public_key: &str,
) -> Result<bool> {
    let key_bytes = decode_fixed_base64::<32>("Ed25519 public key", public_key)?;
    let signature_bytes = decode_signature_bytes(signature)?;
    let verifying_key =
        VerifyingKey::from_bytes(&key_bytes).context("invalid Ed25519 public key")?;
    let signature = Signature::from_bytes(&signature_bytes);
    Ok(verifying_key.verify(payload, &signature).is_ok())
}

pub(in crate::snapshots) fn decode_fixed_base64<const N: usize>(
    label: &str,
    value: &str,
) -> Result<[u8; N]> {
    let normalized = value.split_whitespace().collect::<String>();
    let decoded = BASE64_STANDARD
        .decode(normalized.as_bytes())
        .with_context(|| format!("{label} must be base64"))?;
    decoded
        .try_into()
        .map_err(|bytes: Vec<u8>| anyhow::anyhow!("{label} must be {N} bytes, got {}", bytes.len()))
}

fn decode_signature_bytes(raw: &[u8]) -> Result<[u8; 64]> {
    if raw.len() == 64 {
        return raw
            .try_into()
            .map_err(|_| anyhow::anyhow!("snapshot catalog signature must be 64 bytes"));
    }

    let text = std::str::from_utf8(raw)
        .context("snapshot catalog signature must be raw bytes or base64 text")?;
    decode_fixed_base64::<64>("snapshot catalog signature", text)
}
