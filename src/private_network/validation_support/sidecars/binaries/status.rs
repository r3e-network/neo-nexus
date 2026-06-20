use super::super::super::super::*;
use super::{
    super::super::checks::add_check,
    super::super::paths::{
        resolve_launch_pack_reference, signer_binary_should_search_path,
        signer_sidecar_binary_is_foreign,
    },
    executable::executable_file_status,
    lookup::find_signer_binary_on_path,
};

pub(in crate::private_network) fn check_signer_sidecar_binary(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    signer: &DeploymentCommitteeSignerManifest,
    plan: &SignerCommandPlan,
) {
    let (status, message) = signer_sidecar_binary_status(root_path, &plan.binary);
    add_check(
        checks,
        "signer-sidecar-binary",
        &signer.label,
        status,
        message,
    );
}

pub(in crate::private_network::validation_support::sidecars) fn signer_sidecar_process_binary_path(
    root_path: &Path,
    binary: &str,
) -> PathBuf {
    let binary_path = Path::new(binary);
    if signer_binary_should_search_path(binary_path, binary) {
        PathBuf::from(binary)
    } else {
        resolve_launch_pack_reference(root_path, binary)
    }
}

fn signer_sidecar_binary_status(
    root_path: &Path,
    binary: &str,
) -> (LaunchPackValidationStatus, String) {
    if binary.trim().is_empty() {
        return (
            LaunchPackValidationStatus::Fail,
            "sidecar binary is empty".to_string(),
        );
    }
    if signer_sidecar_binary_is_foreign(binary) {
        return (
            LaunchPackValidationStatus::Warn,
            format!("foreign-platform sidecar binary path not checked on this host: {binary}"),
        );
    }
    if signer_binary_should_search_path(Path::new(binary), binary) {
        return signer_sidecar_path_lookup_status(binary);
    }

    let resolved = resolve_launch_pack_reference(root_path, binary);
    executable_file_status(
        &resolved,
        format!("sidecar binary exists at {}", resolved.display()),
    )
}

fn signer_sidecar_path_lookup_status(binary: &str) -> (LaunchPackValidationStatus, String) {
    find_signer_binary_on_path(binary).map_or_else(
        || {
            (
                LaunchPackValidationStatus::Fail,
                format!("sidecar binary was not found on PATH: {binary}"),
            )
        },
        |resolved| {
            executable_file_status(
                &resolved,
                format!(
                    "sidecar binary resolves through PATH to {}",
                    resolved.display()
                ),
            )
        },
    )
}
