use std::path::Path;

use crate::app::domain::{CommitteeSidecarProcess, PrivateNetworkLaunchPackSidecarReport};

use super::SidecarExecutionPolicyFinding;

pub(in crate::app) fn sidecar_execution_policy_label(allow_external: bool) -> &'static str {
    if allow_external {
        "external allowed"
    } else {
        "bundled only"
    }
}

pub(in crate::app) fn sidecar_execution_policy_findings(
    report: &PrivateNetworkLaunchPackSidecarReport,
    allow_external: bool,
) -> Vec<SidecarExecutionPolicyFinding> {
    report
        .sidecars
        .iter()
        .filter_map(|sidecar| sidecar_execution_policy_finding(report, sidecar, allow_external))
        .collect()
}

pub(in crate::app) fn sidecar_execution_policy_finding(
    report: &PrivateNetworkLaunchPackSidecarReport,
    sidecar: &CommitteeSidecarProcess,
    allow_external: bool,
) -> Option<SidecarExecutionPolicyFinding> {
    if allow_external || sidecar_binary_is_bundled(&report.root_path, &sidecar.process.binary_path)
    {
        return None;
    }

    let reason = if sidecar_binary_uses_path_lookup(&sidecar.process.binary_path) {
        "PATH lookup is external to the launch pack".to_string()
    } else {
        format!(
            "binary is outside launch pack root {}",
            report.root_path.display()
        )
    };

    Some(SidecarExecutionPolicyFinding {
        signer_label: sidecar.signer_label.clone(),
        process_id: sidecar.process.id.clone(),
        binary_path: sidecar.process.binary_path.clone(),
        reason,
    })
}

fn sidecar_binary_is_bundled(root_path: &Path, binary_path: &Path) -> bool {
    if sidecar_binary_uses_path_lookup(binary_path) {
        return false;
    }
    binary_path.starts_with(root_path)
}

fn sidecar_binary_uses_path_lookup(binary_path: &Path) -> bool {
    !binary_path.is_absolute() && binary_path.components().count() == 1
}
