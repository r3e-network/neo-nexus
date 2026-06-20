use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{
    snapshots::{normalize_sha256, sha256_file},
    types::{NodeConfig, NodeType},
};

use super::{
    archive::unpack_plugin_zip,
    fs_utils::{
        backup_dir, ensure_real_directory_exists, replace_plugin_directory, reset_directory,
        staging_dir,
    },
    model::{PluginInstallation, PluginPackageManifest},
    validation::{validate_plugin_package_manifest, verified_plugin_source},
    PLUGIN_CONTROL_DIR, PLUGIN_PACKAGE_MAX_BYTES,
};

mod clock;
mod manifest;

use clock::current_unix_time;
use manifest::{write_install_manifest, InstallManifestRequest};

pub struct PluginPackageManager;

impl PluginPackageManager {
    pub fn install(
        manifest: &PluginPackageManifest,
        node: &NodeConfig,
        node_work_dir: impl AsRef<Path>,
    ) -> Result<PluginInstallation> {
        validate_plugin_package_manifest(manifest)?;
        if node.node_type != NodeType::NeoCli {
            anyhow::bail!("plugin packages are supported for neo-cli nodes only");
        }

        let source_path = verified_plugin_source(&manifest.source_path)?;
        let metadata = fs::metadata(&source_path).with_context(|| {
            format!("failed to inspect plugin package {}", source_path.display())
        })?;
        if metadata.len() > PLUGIN_PACKAGE_MAX_BYTES {
            anyhow::bail!(
                "plugin package is too large: {} bytes exceeds limit {}",
                metadata.len(),
                PLUGIN_PACKAGE_MAX_BYTES
            );
        }

        let expected_sha256 = normalize_sha256(&manifest.expected_sha256)?;
        let (sha256, package_bytes) = sha256_file(&source_path)?;
        if sha256 != expected_sha256 {
            anyhow::bail!(
                "plugin package checksum mismatch: expected {expected_sha256}, got {sha256}"
            );
        }

        let installed_at_unix = current_unix_time()?;
        let node_work_dir = node_work_dir.as_ref();
        ensure_real_directory_exists(node_work_dir, "node work directory")?;
        let plugins_root = node_work_dir.join("Plugins");
        ensure_real_directory_exists(&plugins_root, "neo-cli plugins directory")?;

        let plugin_dir_name = manifest.plugin_id.to_string();
        let target_dir = plugins_root.join(&plugin_dir_name);
        let control_root = plugins_root.join(PLUGIN_CONTROL_DIR);
        ensure_real_directory_exists(&control_root, "NeoNexus plugin control directory")?;
        let staging_dir = staging_dir(&control_root, &plugin_dir_name, installed_at_unix);
        let backup_dir = backup_dir(&control_root, &plugin_dir_name, installed_at_unix);

        reset_directory(&staging_dir)?;
        let install_result = match unpack_plugin_zip(&source_path, &staging_dir) {
            Ok(result) => result,
            Err(error) => {
                let _ = fs::remove_dir_all(&staging_dir);
                return Err(error);
            }
        };
        if install_result.installed_files == 0 {
            let _ = fs::remove_dir_all(&staging_dir);
            anyhow::bail!("plugin package did not contain installable files");
        }

        write_install_manifest(InstallManifestRequest {
            manifest,
            node,
            source_path: &source_path,
            sha256: &sha256,
            package_bytes,
            install_result: &install_result,
            installed_at_unix,
            staging_dir: &staging_dir,
        })?;

        if let Err(error) = replace_plugin_directory(&staging_dir, &target_dir, &backup_dir) {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(error);
        }

        Ok(PluginInstallation {
            node_id: node.id.clone(),
            plugin_id: manifest.plugin_id,
            installed_path: target_dir.clone(),
            manifest_path: target_dir.join(PLUGIN_CONTROL_DIR).join("manifest.json"),
            source_path,
            sha256,
            package_bytes,
            installed_files: install_result.installed_files,
            expanded_bytes: install_result.expanded_bytes,
            installed_at_unix,
        })
    }
}
