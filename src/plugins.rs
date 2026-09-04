mod activation;
mod archive;
mod configuration;
mod fs_utils;
mod manager;
mod model;
mod release;
mod signclient;
mod validation;

pub use manager::PluginPackageManager;
pub use model::{PluginInstallation, PluginPackageManifest};
pub use validation::validate_plugin_package_manifest;

pub(super) const PLUGIN_PACKAGE_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub(super) const PLUGIN_PACKAGE_MAX_EXPANDED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub(super) const PLUGIN_PACKAGE_MAX_FILES: usize = 20_000;
pub(super) const PLUGIN_CONTROL_DIR: &str = ".neonexus";

pub use release::{installed_plugin_release, PluginReleaseMetadata};
pub use signclient::SignClientSettings;
