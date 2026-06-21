use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn apply_selected_runtime_to_node(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.notice = Some("Select a node before applying a runtime".to_string());
            return;
        };
        if node.status.is_active() {
            self.notice = Some("Stop the selected node before applying a runtime".to_string());
            return;
        }

        let installations = self.runtime_installations();
        let Some(installation) = self.selected_runtime_installation(&installations) else {
            self.notice = Some("Select an installed runtime first".to_string());
            return;
        };
        if installation.node_type != node.node_type {
            self.notice = Some(format!(
                "{} cannot use a {} runtime",
                node.name, installation.node_type
            ));
            return;
        }

        let input = NewNode {
            name: node.name.clone(),
            node_type: node.node_type,
            network: node.network,
            binary_path: installation.binary_path.clone(),
            args: node.args.clone(),
            runtime_version: installation.version.clone(),
            storage_engine: node.storage_engine,
            rpc_port: node.rpc_port,
            p2p_port: node.p2p_port,
            ws_port: node.ws_port,
        };

        match self.repository.update_node(&node.id, input) {
            Ok(updated) => {
                let message = format!(
                    "{} now uses {} {} at {}",
                    updated.name,
                    installation.node_type,
                    installation.version,
                    short_path(&installation.binary_path, 54)
                );
                self.record_node_event(
                    &updated,
                    EventKind::RuntimeApplied,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.notice = Some(message);
                self.reload_nodes();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
