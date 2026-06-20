use super::super::*;
use super::{
    PrivateNetworkDeploymentExport, PrivateNetworkDeploymentExporter,
    PrivateNetworkDeploymentRequest,
};

mod artifacts;
mod assembly;
mod context;
mod files;
mod nodes;

impl PrivateNetworkDeploymentExporter {
    pub fn write(
        request: PrivateNetworkDeploymentRequest,
    ) -> Result<PrivateNetworkDeploymentExport> {
        validate_request(&request)?;

        let context = context::LaunchPackWriteContext::prepare(&request)?;
        let node_report = nodes::write_node_configs(&request, &context)?;
        let mut manifest = assembly::deployment_manifest(&request, &context, node_report.nodes);
        let texts = artifacts::render_launch_pack_texts(&manifest)?;
        artifacts::attach_artifact_manifests(&mut manifest, &texts);
        let file_report = files::write_launch_pack_files(&context.root_path, &manifest, &texts)?;

        Ok(PrivateNetworkDeploymentExport {
            root_path: context.root_path,
            manifest_path: file_report.manifest_path,
            start_order_path: file_report.start_order_path,
            runbook_path: file_report.runbook_path,
            wallet_provisioning_path: file_report.wallet_provisioning_path,
            wallet_instructions_path: file_report.wallet_instructions_path,
            preflight_unix_path: file_report.preflight_unix_path,
            preflight_windows_path: file_report.preflight_windows_path,
            health_unix_path: file_report.health_unix_path,
            health_windows_path: file_report.health_windows_path,
            start_unix_path: file_report.start_unix_path,
            stop_unix_path: file_report.stop_unix_path,
            start_windows_path: file_report.start_windows_path,
            stop_windows_path: file_report.stop_windows_path,
            node_count: request.plan.nodes.len(),
            config_count: node_report.config_count,
            network_magic: context.network_magic,
            validators_count: context.validators_count,
            bytes_written: node_report.bytes_written + file_report.bytes_written,
        })
    }
}
