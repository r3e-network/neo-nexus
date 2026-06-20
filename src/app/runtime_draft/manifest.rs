use std::path::PathBuf;

use anyhow::Result;

use crate::runtime::{RuntimePackageManifest, RuntimePlatform};

use super::{optional_string, RuntimePackageDraft};

impl RuntimePackageDraft {
    pub(in crate::app) fn to_manifest(&self) -> Result<RuntimePackageManifest> {
        Ok(RuntimePackageManifest {
            id: self.id.trim().to_string(),
            label: self.label.trim().to_string(),
            node_type: self.node_type,
            version: self.version.trim().to_string(),
            platform: RuntimePlatform {
                os: self.os.trim().to_string(),
                arch: self.arch.trim().to_string(),
            },
            source_path: PathBuf::from(self.source_path.trim()),
            executable_name: self.executable_name.trim().to_string(),
            expected_sha256: self.expected_sha256.trim().to_string(),
            signature_path: optional_path(&self.signature_path),
            ed25519_public_key: optional_string(&self.ed25519_public_key),
        })
    }

    pub(in crate::app) fn use_current_platform(&mut self) {
        let platform = RuntimePlatform::current();
        self.os = platform.os;
        self.arch = platform.arch;
    }
}

fn optional_path(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}
