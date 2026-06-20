use std::{fs, path::Path};

use anyhow::{Context, Result};

use super::super::{
    current_unix_time,
    io::{
        copy_file_hashed, make_executable, replace_file, safe_file_name, safe_fragment,
        verified_source_path,
    },
    RuntimeInstallation, RuntimePackageManifest, RuntimePlatform,
};
use super::RuntimePackageManager;

mod manifest;

use manifest::write_install_manifest;

impl RuntimePackageManager {
    pub fn install(
        manifest: &RuntimePackageManifest,
        install_root: impl AsRef<Path>,
    ) -> Result<RuntimeInstallation> {
        let verification = Self::verify(manifest)?;
        if !verification.matches {
            anyhow::bail!(
                "runtime package checksum mismatch: expected {}, got {}",
                verification.expected_sha256,
                verification.sha256
            );
        }
        if !verification.platform_matches {
            anyhow::bail!(
                "runtime package platform {} does not match current platform {}",
                manifest.platform,
                RuntimePlatform::current()
            );
        }
        if verification.signature_verified == Some(false) {
            anyhow::bail!("runtime package signature verification failed");
        }

        let source = verified_source_path(&manifest.source_path)?;
        let install_dir = install_root
            .as_ref()
            .join(safe_fragment(&manifest.node_type.to_string()))
            .join(safe_fragment(&manifest.version))
            .join(manifest.platform.id());
        fs::create_dir_all(&install_dir).with_context(|| {
            format!(
                "failed to create runtime install directory {}",
                install_dir.display()
            )
        })?;

        let executable_name = safe_file_name(&manifest.executable_name)?;
        let binary_path = install_dir.join(executable_name);
        let temp_path = binary_path.with_extension("installing");
        let (copied_sha256, copied_bytes) = copy_file_hashed(&source, &temp_path)?;
        if copied_sha256 != verification.sha256 || copied_bytes != verification.bytes {
            let _ = fs::remove_file(&temp_path);
            anyhow::bail!("runtime package copy verification failed before install");
        }
        replace_file(&temp_path, &binary_path)?;
        make_executable(&binary_path)?;

        let installed_at_unix = current_unix_time()?;
        let installation = RuntimeInstallation {
            package_id: manifest.id.trim().to_string(),
            label: manifest.label.trim().to_string(),
            node_type: manifest.node_type,
            version: manifest.version.trim().to_string(),
            platform: manifest.platform.clone(),
            binary_path: binary_path.clone(),
            sha256: verification.sha256,
            signature_verified: verification.signature_verified.unwrap_or(false),
            signer_public_key: manifest.ed25519_public_key.clone(),
            bytes: verification.bytes,
            installed_at_unix,
        };
        write_install_manifest(&install_dir, &installation)?;
        Ok(installation)
    }
}
