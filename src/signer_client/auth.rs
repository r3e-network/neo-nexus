//! Signer caller credentials.
//!
//! Authentication keys authorize requests; they are not Neo custody keys.

use std::fmt;

use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

const WORKLOAD_PROTOCOL: &str = "neoos-workload-v1";

#[derive(Clone)]
pub enum SignerCredential {
    Bearer(BearerCredential),
    Workload(Box<WorkloadCredential>),
}

impl fmt::Debug for SignerCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bearer(value) => value.fmt(formatter),
            Self::Workload(value) => value.fmt(formatter),
        }
    }
}

#[derive(Clone)]
pub struct BearerCredential {
    token: Zeroizing<String>,
}

impl BearerCredential {
    pub fn new(token: impl Into<String>) -> Result<Self> {
        let token = Zeroizing::new(token.into());
        if token.trim().is_empty() {
            bail!("signer bearer token is empty");
        }
        if token.contains(['\r', '\n']) {
            bail!("signer bearer token contains a line break");
        }
        Ok(Self { token })
    }
}

impl fmt::Debug for BearerCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BearerCredential([REDACTED])")
    }
}

#[derive(Clone)]
pub struct WorkloadCredential {
    caller_id: String,
    subject: Option<String>,
    signing_key: SigningKey,
}

impl WorkloadCredential {
    pub fn from_seed_hex(
        caller_id: impl Into<String>,
        subject: Option<String>,
        seed_hex: &str,
    ) -> Result<Self> {
        let seed = decode_lower_hex_32(seed_hex.trim())
            .context("workload key must be 64 lowercase hexadecimal characters")?;
        Self::from_seed(caller_id, subject, seed)
    }

    pub fn from_seed(
        caller_id: impl Into<String>,
        subject: Option<String>,
        mut seed: [u8; 32],
    ) -> Result<Self> {
        let caller_id = caller_id.into();
        validate_line_value("workload caller id", &caller_id)?;
        if caller_id.trim().is_empty() {
            bail!("workload caller id is empty");
        }
        if let Some(subject) = subject.as_deref() {
            validate_line_value("workload subject", subject)?;
        }
        let signing_key = SigningKey::from_bytes(&seed);
        seed.zeroize();
        Ok(Self {
            caller_id,
            subject,
            signing_key,
        })
    }

    pub fn caller_id(&self) -> &str {
        &self.caller_id
    }

    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    pub fn public_key_hex(&self) -> String {
        lower_hex(&self.signing_key.verifying_key().to_bytes())
    }
}

impl fmt::Debug for WorkloadCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkloadCredential")
            .field("caller_id", &self.caller_id)
            .field("subject", &self.subject)
            .field("public_key", &self.public_key_hex())
            .field("signing_key", &"[REDACTED]")
            .finish()
    }
}

#[derive(Default)]
pub(crate) struct AuthHeaders {
    pub authorization: Option<Zeroizing<String>>,
    pub caller_id: Option<String>,
    pub timestamp: Option<String>,
    pub nonce: Option<String>,
    pub signature: Option<String>,
}

impl SignerCredential {
    pub(crate) fn headers(
        &self,
        method: &str,
        route: &str,
        body: &[u8],
        timestamp: u64,
        nonce: &str,
    ) -> Result<AuthHeaders> {
        match self {
            Self::Bearer(credential) => Ok(AuthHeaders {
                authorization: Some(Zeroizing::new(format!(
                    "Bearer {}",
                    credential.token.as_str()
                ))),
                ..AuthHeaders::default()
            }),
            Self::Workload(credential) => {
                validate_line_value("workload request route", route)?;
                validate_line_value("workload request nonce", nonce)?;
                let digest = body_sha256(body);
                let message = workload_signing_message(
                    credential.caller_id(),
                    credential.subject(),
                    timestamp,
                    nonce,
                    method,
                    route,
                    &digest,
                );
                let signature = credential.signing_key.sign(&message);
                Ok(AuthHeaders {
                    caller_id: Some(credential.caller_id.clone()),
                    timestamp: Some(timestamp.to_string()),
                    nonce: Some(nonce.to_string()),
                    signature: Some(lower_hex(&signature.to_bytes())),
                    ..AuthHeaders::default()
                })
            }
        }
    }
}

pub fn body_sha256(body: &[u8]) -> [u8; 32] {
    Sha256::digest(body).into()
}

pub fn workload_signing_message(
    caller_id: &str,
    subject: Option<&str>,
    timestamp: u64,
    nonce: &str,
    method: &str,
    route: &str,
    body_digest: &[u8; 32],
) -> Vec<u8> {
    format!(
        "{WORKLOAD_PROTOCOL}\ncaller:{caller_id}\nsubject:{}\ntimestamp:{timestamp}\nnonce:{nonce}\nmethod:{}\nroute:{route}\nbody-sha256:{}\norigin:",
        subject.unwrap_or_default(),
        method.to_ascii_uppercase(),
        lower_hex(body_digest),
    )
    .into_bytes()
}

pub(crate) fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_lower_hex_32(value: &str) -> Result<[u8; 32]> {
    if value.len() != 64 || value.bytes().any(|byte| !byte.is_ascii_hexdigit()) {
        bail!("invalid Ed25519 seed shape");
    }
    if value.bytes().any(|byte| byte.is_ascii_uppercase()) {
        bail!("Ed25519 seed must use lowercase hexadecimal");
    }
    let mut decoded = [0u8; 32];
    for (index, slot) in decoded.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .context("invalid Ed25519 seed")?;
    }
    Ok(decoded)
}

fn validate_line_value(label: &str, value: &str) -> Result<()> {
    if value.contains(['\r', '\n']) {
        bail!("{label} contains a line break");
    }
    Ok(())
}
