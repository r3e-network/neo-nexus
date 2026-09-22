//! Configuration for the dedicated Neo `SecureSign` gRPC service.
//!
//! This is deliberately not a [`crate::signer_client::SignerClient`].  The
//! latter speaks the NeoOS HTTP custody contract; a locally deployed
//! `secure-sign-service-rs` instance speaks the official neo-cli `SignClient`
//! protobuf contract.  Treating a loopback NeoOS URL as this backend makes a
//! node appear configured while no native node can consume it.

use std::env;

use anyhow::{bail, Context, Result};
use p256::elliptic_curve::sec1::ToEncodedPoint as _;
use url::{Host, Url};

pub const LOCAL_SIGNER_ENDPOINT_ENV: &str = "NEONEXUS_LOCAL_SIGNER_ENDPOINT";
pub const LOCAL_SIGNER_PUBLIC_KEY_ENV: &str = "NEONEXUS_LOCAL_SIGNER_PUBLIC_KEY";
pub const LOCAL_SIGNER_NETWORK_MAGIC_ENV: &str = "NEONEXUS_LOCAL_SIGNER_NETWORK_MAGIC";

/// One local consensus signer endpoint and the immutable identity expected
/// behind it.  The public key doubles as the node binding's key id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSignerConfig {
    endpoint: String,
    public_key: String,
    network_magic: u32,
}

impl LocalSignerConfig {
    pub fn new(
        endpoint: impl Into<String>,
        public_key: impl Into<String>,
        network_magic: u32,
    ) -> Result<Self> {
        let endpoint = validate_endpoint(&endpoint.into())?;
        let public_key = normalize_public_key(&public_key.into())?;
        if network_magic == 0 {
            bail!("local signer network magic must be greater than zero");
        }
        Ok(Self {
            endpoint,
            public_key,
            network_magic,
        })
    }

    pub fn from_env() -> Result<Option<Self>> {
        let endpoint = read_env(LOCAL_SIGNER_ENDPOINT_ENV)?;
        let public_key = read_env(LOCAL_SIGNER_PUBLIC_KEY_ENV)?;
        let network_magic = read_env(LOCAL_SIGNER_NETWORK_MAGIC_ENV)?;
        if endpoint.is_none() && public_key.is_none() && network_magic.is_none() {
            return Ok(None);
        }
        let endpoint = required(endpoint, LOCAL_SIGNER_ENDPOINT_ENV)?;
        let public_key = required(public_key, LOCAL_SIGNER_PUBLIC_KEY_ENV)?;
        let network_magic = required(network_magic, LOCAL_SIGNER_NETWORK_MAGIC_ENV)?
            .parse::<u32>()
            .with_context(|| format!("{LOCAL_SIGNER_NETWORK_MAGIC_ENV} must be a u32"))?;
        Self::new(endpoint, public_key, network_magic).map(Some)
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub fn network_magic(&self) -> u32 {
        self.network_magic
    }
}

fn validate_endpoint(raw: &str) -> Result<String> {
    let raw = raw.trim();
    let endpoint = Url::parse(raw).context("local signer endpoint is not a valid URL")?;
    if !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.query().is_some()
        || endpoint.fragment().is_some()
        || !matches!(endpoint.path(), "" | "/")
    {
        bail!(
            "local signer endpoint must be an origin without credentials, path, query, or fragment"
        );
    }
    match endpoint.scheme() {
        "http" | "https" => {
            let loopback = match endpoint.host() {
                Some(Host::Ipv4(ip)) => ip.is_loopback(),
                Some(Host::Ipv6(ip)) => ip.is_loopback(),
                Some(Host::Domain(_)) => false,
                None => bail!("local signer TCP endpoint requires a host"),
            };
            if !loopback {
                bail!("local signer TCP endpoint must use a loopback address");
            }
            if endpoint.port().is_none() {
                bail!("local signer TCP endpoint requires an explicit port");
            }
        }
        "vsock" => {
            let context_id = endpoint
                .host_str()
                .context("local signer vsock endpoint requires a context id")?;
            context_id
                .parse::<u32>()
                .context("local signer vsock context id must be a u32")?;
            if endpoint.port().is_none() {
                bail!("local signer vsock endpoint requires an explicit port");
            }
        }
        other => bail!(
            "local signer endpoint scheme {other:?} is unsupported; use http, https, or vsock"
        ),
    }
    Ok(raw.to_string())
}

fn normalize_public_key(raw: &str) -> Result<String> {
    let raw = raw.trim();
    let bytes = decode_hex(raw).context("local signer public key must be hexadecimal")?;
    if bytes.len() != 33 {
        bail!("local signer public key must be a 33-byte compressed P-256 SEC1 key");
    }
    let key = p256::PublicKey::from_sec1_bytes(&bytes)
        .context("local signer public key is not a valid P-256 point")?;
    Ok(encode_hex(key.to_encoded_point(true).as_bytes()))
}

fn decode_hex(value: &str) -> Result<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        bail!("hexadecimal text has an odd length");
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let high = digit(pair[0])?;
            let low = digit(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn digit(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => bail!("invalid hexadecimal digit"),
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}

fn read_env(name: &str) -> Result<Option<String>> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => bail!("{name} must contain valid Unicode"),
    }
}

fn required(value: Option<String>, name: &str) -> Result<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .with_context(|| format!("{name} is required for the local-signer backend"))
}

#[cfg(test)]
#[path = "../../tests/unit/signing/local_signer/tests.rs"]
mod tests;
