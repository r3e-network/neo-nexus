use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn auto_assign_draft_ports(&mut self) {
        let preferred_rpc = self.fleet
            .draft
            .rpc_port
            .trim()
            .parse::<u16>()
            .unwrap_or(DEFAULT_RPC_PORT);
        let include_ws = !self.fleet.draft.ws_port.trim().is_empty();

        match plan_available_node_ports(&self.fleet.nodes, None, preferred_rpc, include_ws) {
            Ok(assignment) => {
                self.apply_port_assignment_to_draft(assignment);
                self.session.notice = Some(format!("Draft ports assigned: {}", assignment.summary()));
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn assign_available_ports_to_selected_node(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.session.notice = Some("Select a node before assigning ports".to_string());
            return;
        };

        if node.status.is_active() {
            self.session.notice = Some("Stop the selected node before assigning ports".to_string());
            return;
        }

        let include_ws = node.ws_port.is_some();
        match plan_available_node_ports(&self.fleet.nodes, Some(&node.id), node.rpc_port, include_ws) {
            Ok(assignment) => {
                if node.rpc_port == assignment.rpc_port
                    && node.p2p_port == assignment.p2p_port
                    && node.ws_port == assignment.ws_port
                {
                    self.session.notice = Some(format!(
                        "{} already uses an available port block: {}",
                        node.name,
                        assignment.summary()
                    ));
                    return;
                }

                let input = NewNode {
                    name: node.name.clone(),
                    node_type: node.node_type,
                    network: node.network,
                    binary_path: node.binary_path.clone(),
                    args: node.args.clone(),
                    runtime_version: node.runtime_version.clone(),
                    storage_engine: node.storage_engine,
                    rpc_port: assignment.rpc_port,
                    p2p_port: assignment.p2p_port,
                    ws_port: assignment.ws_port,
                };

                match self.repository.update_node(&node.id, input) {
                    Ok(updated) => {
                        let message =
                            format!("{} ports assigned: {}", updated.name, assignment.summary());
                        self.record_node_event(
                            &updated,
                            EventKind::NodePortsAssigned,
                            EventSeverity::Info,
                            message.clone(),
                        );
                        self.fleet.selected_node = Some(updated.id.clone());
                        self.session.notice = Some(message);
                        self.reload_nodes();
                    }
                    Err(error) => self.session.notice = Some(error.to_string()),
                }
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn apply_port_assignment_to_draft(&mut self, assignment: PortAssignment) {
        self.fleet.draft.rpc_port = assignment.rpc_port.to_string();
        self.fleet.draft.p2p_port = assignment.p2p_port.to_string();
        self.fleet.draft.ws_port = assignment
            .ws_port
            .map_or_else(String::new, |port| port.to_string());
    }
}
