use anyhow::Result;

use crate::snapshots::{normalize_sha256, sha256_file};

use super::super::{
    current_unix_time, io::verified_source_path, security::verify_runtime_signature,
    validate_runtime_manifest, RuntimePackageManifest, RuntimePackageVerification, RuntimePlatform,
};
use super::RuntimePackageManager;

impl RuntimePackageManager {
    pub fn verify(manifest: &RuntimePackageManifest) -> Result<RuntimePackageVerification> {
        validate_runtime_manifest(manifest)?;
        let source = verified_source_path(&manifest.source_path)?;
        let (sha256, bytes) = sha256_file(&source)?;
        let expected_sha256 = normalize_sha256(&manifest.expected_sha256)?;
        let signature_verified = verify_runtime_signature(manifest, &source).transpose()?;
        Ok(RuntimePackageVerification {
            matches: sha256 == expected_sha256,
            sha256,
            expected_sha256,
            bytes,
            platform_matches: manifest.platform == RuntimePlatform::current(),
            signature_verified,
            verified_at_unix: current_unix_time()?,
        })
    }
}
