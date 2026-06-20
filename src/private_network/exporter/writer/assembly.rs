use super::{context::LaunchPackWriteContext, *};

pub(super) fn deployment_manifest(
    request: &PrivateNetworkDeploymentRequest,
    context: &LaunchPackWriteContext,
    nodes: Vec<DeploymentNodeManifest>,
) -> DeploymentManifest {
    DeploymentManifest {
        schema_version: LAUNCH_PACK_SCHEMA_VERSION,
        generated_at_unix: context.generated_at_unix,
        template: request.plan.template.label().to_string(),
        runtime: request.plan.node_type.to_string(),
        network: Network::Private.to_string(),
        network_magic: context.network_magic,
        validators_count: context.validators_count,
        seed_nodes: context.seed_nodes.clone(),
        committee: committee_manifest(request.committee.as_ref()),
        secret_provisioning: secret_provisioning_manifest(request.committee.as_ref()),
        scripts: DeploymentScriptsManifest {
            runbook: "RUNBOOK.md".to_string(),
            preflight_unix: "preflight-unix.sh".to_string(),
            preflight_windows: "preflight-windows.ps1".to_string(),
            health_unix: "health-unix.sh".to_string(),
            health_windows: "health-windows.ps1".to_string(),
            start_unix: "start-unix.sh".to_string(),
            stop_unix: "stop-unix.sh".to_string(),
            start_windows: "start-windows.ps1".to_string(),
            stop_windows: "stop-windows.ps1".to_string(),
        },
        artifacts: Vec::new(),
        nodes,
    }
}
