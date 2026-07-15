use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn probe_draft_binary(&mut self) {
        let input = match self.fleet.draft.to_new_node() {
            Ok(input) => input,
            Err(error) => {
                self.session.notice = Some(error.to_string());
                return;
            }
        };

        let node = NodeConfig {
            id: "draft".to_string(),
            name: input.name,
            node_type: input.node_type,
            network: input.network,
            binary_path: input.binary_path,
            args: input.args,
            runtime_version: input.runtime_version,
            storage_engine: input.storage_engine,
            rpc_port: input.rpc_port,
            p2p_port: input.p2p_port,
            ws_port: input.ws_port,
            status: NodeStatus::Stopped,
            pid: None,
        };
        let report = inspect_node_binary(&node);
        self.session.notice = Some(preflight_notice(&report));
    }

    pub(in crate::app) fn probe_selected_binary(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.session.notice = Some("Select a node before probing its binary".to_string());
            return;
        };

        let report = inspect_node_binary(&node);
        self.session.notice = Some(preflight_notice(&report));
    }

    pub(in crate::app) fn smoke_draft_runtime(&mut self) {
        let input = match self.fleet.draft.to_new_node() {
            Ok(input) => input,
            Err(error) => {
                self.session.notice = Some(error.to_string());
                return;
            }
        };

        let node = NodeConfig {
            id: "draft".to_string(),
            name: input.name,
            node_type: input.node_type,
            network: input.network,
            binary_path: input.binary_path,
            args: input.args,
            runtime_version: input.runtime_version,
            storage_engine: input.storage_engine,
            rpc_port: input.rpc_port,
            p2p_port: input.p2p_port,
            ws_port: input.ws_port,
            status: NodeStatus::Stopped,
            pid: None,
        };
        let report = smoke_node_binary(&node, RUNTIME_SMOKE_TIMEOUT);
        self.session.notice = Some(runtime_smoke_notice(&report));
    }

    pub(in crate::app) fn smoke_selected_runtime(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.session.notice = Some("Select a node before smoke testing its runtime".to_string());
            return;
        };

        let report = smoke_node_binary(&node, RUNTIME_SMOKE_TIMEOUT);
        let message = runtime_smoke_notice(&report);
        self.record_node_event(
            &node,
            EventKind::RuntimeSmokeTested,
            runtime_smoke_event_severity(report.status),
            message.clone(),
        );
        self.session.notice = Some(message);
    }
}
