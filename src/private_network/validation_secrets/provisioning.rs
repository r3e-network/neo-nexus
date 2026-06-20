mod files;
mod metadata;
mod warnings;

use self::{
    files::check_secret_provisioning_files, metadata::check_secret_provisioning_metadata,
    warnings::warn_missing_wallet_references,
};
use super::*;

pub(in crate::private_network) fn check_secret_provisioning(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    manifest: &DeploymentManifest,
) {
    let provisioning = &manifest.secret_provisioning;
    check_secret_provisioning_metadata(checks, manifest, provisioning);
    check_secret_provisioning_files(checks, root_path, manifest, provisioning);
    warn_missing_wallet_references(checks, manifest);
}
