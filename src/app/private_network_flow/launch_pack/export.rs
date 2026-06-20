use super::*;

use crate::private_network::PrivateNetworkDeploymentExport;

impl NeoNexusApp {
    pub(in crate::app) fn export_private_network_launch_pack(&mut self) {
        let plan = PrivateNetworkPlanner::plan(
            self.private_network_template,
            self.private_network_runtime,
        );
        let nodes = match self.private_network_materialized_nodes(&plan) {
            Ok(nodes) => nodes,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };
        if let Some(message) = launch_pack_running_node_notice(&nodes) {
            self.notice = Some(message);
            return;
        }

        let committee = match CommitteeRoster::from_public_keys_and_references(
            &self.private_network_committee_keys,
            &self.private_network_signer_refs,
        ) {
            Ok(committee) => committee,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };
        let plugin_states = self.private_launch_pack_plugin_states(&nodes);
        let request = PrivateNetworkDeploymentRequest {
            plan,
            nodes,
            plugin_states,
            committee,
            output_dir: self.private_network_export_dir(),
            node_root_dir: self.node_root_dir(),
        };

        match PrivateNetworkDeploymentExporter::write(request) {
            Ok(export) => self.record_private_launch_pack_export(export),
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    fn private_launch_pack_plugin_states(
        &self,
        nodes: &[NodeConfig],
    ) -> BTreeMap<String, Vec<PluginState>> {
        nodes
            .iter()
            .map(|node| {
                (
                    node.id.clone(),
                    self.repository
                        .list_plugin_states(&node.id)
                        .unwrap_or_default(),
                )
            })
            .collect()
    }

    fn record_private_launch_pack_export(&mut self, export: PrivateNetworkDeploymentExport) {
        let export_path = short_path(&export.root_path, 54);
        let export_message = format!(
            "Private launch pack exported: {} nodes, magic {}, {}",
            export.node_count, export.network_magic, export_path
        );
        self.private_network_last_export_root = Some(export.root_path.clone());
        self.record_event(
            None,
            None,
            EventKind::PrivateNetworkLaunchPackExported,
            EventSeverity::Info,
            export_message.clone(),
        );
        let validation_message = self.validate_private_network_launch_pack(&export.root_path);
        self.notice = Some(format!("{export_message}; {validation_message}"));
    }
}

fn launch_pack_running_node_notice(nodes: &[NodeConfig]) -> Option<String> {
    nodes
        .iter()
        .find(|node| matches!(node.status, NodeStatus::Running | NodeStatus::Starting))
        .map(|node| format!("Stop {} before exporting a launch pack", node.name))
}
