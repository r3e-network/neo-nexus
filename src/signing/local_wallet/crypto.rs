use std::fmt::Write as _;

use aes::cipher::{BlockDecrypt, KeyInit};
use anyhow::{bail, Context, Result};
use p256::ecdsa::{
    signature::{Signer as _, Verifier as _},
    Signature as P256Signature, SigningKey,
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::{
    signer_client::KeyPublic,
    wallet::crypto::{
        base58check_encode, base58check_payload, double_sha256, hash160, valid_scrypt_parameters,
    },
};

use super::document::WalletScrypt;

const ADDRESS_VERSION: u8 = 0x35;
const MAX_SIGNING_BYTES: usize = 256 * 1024;

pub(super) fn decrypt_nep2(
    value: &str,
    passphrase: &str,
    wallet_scrypt: WalletScrypt,
) -> Result<Zeroizing<[u8; 32]>> {
    let value = value.trim();
    if value.len() != 58 || !value.is_ascii() || !value.starts_with("6P") {
        bail!("the account key is not a bounded NEP-2 value");
    }
    let payload = base58check_payload(value).context("the account key is not Base58Check")?;
    if payload.len() != 39 || payload.get(0..3) != Some(&[0x01, 0x42, 0xe0]) {
        bail!("the account key is not a supported non-EC-multiply NEP-2 key");
    }
    let address_hash = payload
        .get(3..7)
        .context("the NEP-2 address hash is missing")?;
    let encrypted = payload
        .get(7..39)
        .context("the NEP-2 encrypted key is missing")?;
    let mut derived = Zeroizing::new([0_u8; 64]);
    if !valid_scrypt_parameters(
        wallet_scrypt.n,
        u64::from(wallet_scrypt.r),
        u64::from(wallet_scrypt.p),
    ) {
        bail!("the NEP-6 scrypt parameters are outside the local signing bounds");
    }
    let log_n = u8::try_from(wallet_scrypt.n.trailing_zeros())
        .map_err(|_| anyhow::anyhow!("the NEP-6 scrypt work factor is invalid"))?;
    let params = scrypt::Params::new(log_n, wallet_scrypt.r, wallet_scrypt.p, 64)
        .map_err(|_| anyhow::anyhow!("the NEP-6 scrypt parameters are invalid"))?;
    scrypt::scrypt(passphrase.as_bytes(), address_hash, &params, &mut *derived)
        .context("NEP-2 key derivation failed")?;
    let mut encrypted_key = Zeroizing::new([0_u8; 32]);
    encrypted_key.copy_from_slice(encrypted);
    // NEP-2 encrypts the two 16-byte blocks independently with AES-256-ECB.
    // CBC happens to round-trip against a mirrored test helper, but it cannot
    // open a wallet produced by neo-cli.
    let cipher = aes::Aes256::new_from_slice(&derived[32..])
        .map_err(|_| anyhow::anyhow!("NEP-2 AES parameters are invalid"))?;
    for block in encrypted_key.as_chunks_mut::<16>().0 {
        cipher.decrypt_block(block.into());
    }
    let mut private_key = Zeroizing::new([0_u8; 32]);
    for index in 0..32 {
        private_key[index] = encrypted_key[index] ^ derived[index];
    }
    let identity = identity_parts(&private_key)?;
    let expected_hash = double_sha256(identity.address.as_bytes());
    if expected_hash.get(..4) != Some(address_hash) {
        bail!("the local wallet password is wrong or the NEP-2 key address hash does not match");
    }
    Ok(private_key)
}

pub(super) fn identity_from_private(
    private_key: &[u8; 32],
    network: &str,
    network_magic: u32,
    wallet_sha256: &str,
) -> Result<KeyPublic> {
    let parts = identity_parts(private_key)?;
    Ok(KeyPublic {
        key_id: format!("wallet-{}", parts.address),
        label: format!("Local wallet · {}", parts.address),
        network: network.to_string(),
        network_magic: Some(network_magic),
        chain_family: Some("neo-n3".to_string()),
        chain_id: None,
        public_key: hex_encode(&parts.public_key),
        script_hash: display_hash(&parts.script_hash, true),
        address: parts.address,
        verification_script: hex_encode(&parts.verification_script),
        signing_enabled: true,
        additional_fields: std::collections::BTreeMap::from([
            (
                "backend".to_string(),
                serde_json::Value::String("local-wallet".to_string()),
            ),
            (
                "wallet_sha256".to_string(),
                serde_json::Value::String(wallet_sha256.to_string()),
            ),
        ]),
    })
}

pub(super) fn sign_transaction(
    private_key: &[u8; 32],
    unsigned: &[u8],
    network_magic: u32,
) -> Result<[u8; 64]> {
    let mut preimage = Vec::with_capacity(36);
    preimage.extend_from_slice(&network_magic.to_le_bytes());
    preimage.extend_from_slice(&Sha256::digest(unsigned));
    sign_message(private_key, &preimage)
}

pub(super) fn sign_message(private_key: &[u8; 32], message: &[u8]) -> Result<[u8; 64]> {
    let signing_key = SigningKey::from_slice(private_key).map_err(|_| {
        anyhow::anyhow!("the decrypted local wallet key is not a valid P-256 scalar")
    })?;
    let signature: P256Signature = signing_key.sign(message);
    let signature = signature.normalize_s().unwrap_or(signature);
    signing_key
        .verifying_key()
        .verify(message, &signature)
        .map_err(|_| anyhow::anyhow!("the local wallet signature failed self-verification"))?;
    let mut bytes = [0_u8; 64];
    bytes.copy_from_slice(&signature.to_bytes());
    Ok(bytes)
}

pub(super) fn invocation_script(signature: &[u8; 64]) -> Vec<u8> {
    let mut script = Vec::with_capacity(66);
    script.extend_from_slice(&[0x0c, 0x40]);
    script.extend_from_slice(signature);
    script
}

pub(super) fn decode_request_hex(value: &str) -> Result<Vec<u8>> {
    let encoded = value.trim().strip_prefix("0x").unwrap_or(value.trim());
    if encoded.is_empty() || !encoded.len().is_multiple_of(2) {
        bail!("signing bytes must be non-empty even-length hexadecimal");
    }
    // Validate ASCII before byte-indexing the string below. A non-ASCII value
    // can have an even UTF-8 byte length while a two-byte slice still lands in
    // the middle of a code point.
    if !encoded.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("signing bytes contain non-hexadecimal characters");
    }
    if encoded.len() / 2 > MAX_SIGNING_BYTES {
        bail!("signing bytes exceed the {MAX_SIGNING_BYTES}-byte local wallet limit");
    }
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for index in (0..encoded.len()).step_by(2) {
        let byte = u8::from_str_radix(&encoded[index..index + 2], 16).with_context(|| {
            format!(
                "signing bytes contain invalid hexadecimal at byte {}",
                index / 2
            )
        })?;
        bytes.push(byte);
    }
    Ok(bytes)
}

pub(super) fn transaction_hash_display(unsigned: &[u8]) -> String {
    display_hash(&Sha256::digest(unsigned), true)
}

pub(super) fn sha256_display(bytes: &[u8], reverse: bool, prefix: bool) -> String {
    let digest = Sha256::digest(bytes);
    if reverse {
        display_hash(&digest, prefix)
    } else if prefix {
        format!("0x{}", hex_encode(&digest))
    } else {
        hex_encode(&digest)
    }
}

pub(super) fn hex_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

struct IdentityParts {
    public_key: [u8; 33],
    verification_script: Vec<u8>,
    script_hash: Vec<u8>,
    address: String,
}

fn identity_parts(private_key: &[u8; 32]) -> Result<IdentityParts> {
    let signing_key = SigningKey::from_slice(private_key).map_err(|_| {
        anyhow::anyhow!("the decrypted local wallet key is not a valid P-256 scalar")
    })?;
    let encoded = signing_key.verifying_key().to_encoded_point(true);
    let public_bytes = encoded.as_bytes();
    if public_bytes.len() != 33 {
        bail!("the local wallet P-256 public key is not compressed to 33 bytes");
    }
    let mut public_key = [0_u8; 33];
    public_key.copy_from_slice(public_bytes);
    let mut verification_script = Vec::with_capacity(40);
    verification_script.extend_from_slice(&[0x0c, 0x21]);
    verification_script.extend_from_slice(&public_key);
    verification_script.extend_from_slice(&[0x41, 0x56, 0xe7, 0xb3, 0x27]);
    let script_hash = hash160(&verification_script);
    let mut address_payload = Vec::with_capacity(21);
    address_payload.push(ADDRESS_VERSION);
    address_payload.extend_from_slice(&script_hash);
    Ok(IdentityParts {
        public_key,
        verification_script,
        script_hash,
        address: base58check_encode(&address_payload),
    })
}

fn display_hash(bytes: &[u8], prefix: bool) -> String {
    let mut reversed = bytes.to_vec();
    reversed.reverse();
    let encoded = hex_encode(&reversed);
    if prefix {
        format!("0x{encoded}")
    } else {
        encoded
    }
}
