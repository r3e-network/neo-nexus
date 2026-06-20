use std::{fmt, path::PathBuf};

use crate::types::NodeType;

use super::io::safe_fragment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlatform {
    pub os: String,
    pub arch: String,
}

impl RuntimePlatform {
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }

    pub fn id(&self) -> String {
        format!("{}-{}", safe_fragment(&self.os), safe_fragment(&self.arch))
    }
}

impl fmt::Display for RuntimePlatform {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.id())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePackageManifest {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub version: String,
    pub platform: RuntimePlatform,
    pub source_path: PathBuf,
    pub executable_name: String,
    pub expected_sha256: String,
    pub signature_path: Option<PathBuf>,
    pub ed25519_public_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDownloadRequest {
    pub url: String,
    pub file_name: String,
    pub expected_sha256: String,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSignerProfile {
    pub id: String,
    pub label: String,
    pub ed25519_public_key: String,
    pub enabled: bool,
    pub created_at_unix: u64,
    pub last_used_at_unix: Option<u64>,
}

impl RuntimeSignerProfile {
    pub fn public_key_if_enabled(&self) -> Option<&str> {
        self.enabled
            .then_some(self.ed25519_public_key.trim())
            .filter(|value| !value.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePackageVerification {
    pub sha256: String,
    pub expected_sha256: String,
    pub bytes: u64,
    pub matches: bool,
    pub platform_matches: bool,
    pub signature_verified: Option<bool>,
    pub verified_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDownload {
    pub url: String,
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
    pub downloaded_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInstallation {
    pub package_id: String,
    pub label: String,
    pub node_type: NodeType,
    pub version: String,
    pub platform: RuntimePlatform,
    pub binary_path: PathBuf,
    pub sha256: String,
    pub signature_verified: bool,
    pub signer_public_key: Option<String>,
    pub bytes: u64,
    pub installed_at_unix: u64,
}
