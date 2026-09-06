use std::{fmt, str::FromStr};

use anyhow::{bail, Result};

/// Explicit backend selector.  Location never decides the type implicitly.
pub const BACKEND_ENV: &str = "NEONEXUS_SIGNER_BACKEND";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignerBackendKind {
    LocalWallet,
    LocalSigner,
    NeoOsService,
}

impl SignerBackendKind {
    pub const ALL: [Self; 3] = [Self::LocalWallet, Self::LocalSigner, Self::NeoOsService];

    pub fn slug(self) -> &'static str {
        match self {
            Self::LocalWallet => "local-wallet",
            Self::LocalSigner => "local-signer",
            Self::NeoOsService => "neo-os-service",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::LocalWallet => "Local encrypted wallet",
            Self::LocalSigner => "Locally deployed signer",
            Self::NeoOsService => "NeoOS signer service",
        }
    }

    /// Whether this backend speaks the NeoOS HTTP custody/administration API.
    /// The local signer is a service too, but it speaks the incompatible Neo
    /// `SecureSign` gRPC protocol and must never enter this route.
    pub fn has_http_custody_api(self) -> bool {
        matches!(self, Self::NeoOsService)
    }
}

impl fmt::Display for SignerBackendKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.slug())
    }
}

impl FromStr for SignerBackendKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim() {
            "local-wallet" => Ok(Self::LocalWallet),
            "local-signer" => Ok(Self::LocalSigner),
            "neo-os-service" => Ok(Self::NeoOsService),
            other => bail!(
                "unsupported signer backend {other:?}; expected local-wallet, local-signer, or neo-os-service"
            ),
        }
    }
}

/// Non-secret identity for one configured backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerBackendProfile {
    pub id: String,
    pub label: String,
    pub kind: SignerBackendKind,
}

impl SignerBackendProfile {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        kind: SignerBackendKind,
    ) -> Result<Self> {
        let profile = Self {
            id: id.into().trim().to_string(),
            label: label.into().trim().to_string(),
            kind,
        };
        validate_profile(&profile)?;
        Ok(profile)
    }
}

/// A key name is never meaningful without the backend that owns it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignerKeyRef {
    pub backend_id: String,
    pub key_id: String,
}

impl SignerKeyRef {
    pub fn new(backend_id: impl Into<String>, key_id: impl Into<String>) -> Result<Self> {
        let reference = Self {
            backend_id: backend_id.into().trim().to_string(),
            key_id: key_id.into().trim().to_string(),
        };
        validate_id(&reference.backend_id, "signer backend id")?;
        validate_id(&reference.key_id, "signer key id")?;
        Ok(reference)
    }
}

fn validate_profile(profile: &SignerBackendProfile) -> Result<()> {
    validate_id(&profile.id, "signer backend id")?;
    if profile.label.is_empty() || profile.label.len() > 120 {
        bail!("signer backend label must be 1..=120 characters");
    }
    if profile.label.chars().any(char::is_control) {
        bail!("signer backend label contains control characters");
    }
    Ok(())
}

fn validate_id(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        bail!("{label} must be 1..=128 ASCII alphanumeric, `-`, or `_` characters");
    }
    Ok(())
}
