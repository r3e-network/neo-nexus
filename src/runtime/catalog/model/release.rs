use std::path::PathBuf;

use crate::types::NodeType;

use super::super::super::{RuntimeDownloadRequest, RuntimePackageManifest, RuntimePlatform};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRelease {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub version: String,
    pub platform: RuntimePlatform,
    pub url: String,
    pub file_name: String,
    pub executable_name: String,
    pub expected_sha256: String,
    pub max_bytes: u64,
}

impl RuntimeRelease {
    pub fn download_request(&self) -> RuntimeDownloadRequest {
        RuntimeDownloadRequest {
            url: self.url.clone(),
            file_name: self.file_name.clone(),
            expected_sha256: self.expected_sha256.clone(),
            max_bytes: self.max_bytes,
        }
    }

    pub fn manifest_for_source(&self, source_path: impl Into<PathBuf>) -> RuntimePackageManifest {
        RuntimePackageManifest {
            id: self.id.clone(),
            label: self.label.clone(),
            node_type: self.node_type,
            version: self.version.clone(),
            platform: self.platform.clone(),
            source_path: source_path.into(),
            executable_name: self.executable_name.clone(),
            expected_sha256: self.expected_sha256.clone(),
            signature_path: None,
            ed25519_public_key: None,
        }
    }

    pub fn platform_matches(&self, platform: &RuntimePlatform) -> bool {
        &self.platform == platform
    }
}
