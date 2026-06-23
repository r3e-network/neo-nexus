use super::*;
use crate::app::domain::ProcessStart;

impl NeoNexusApp {
    pub(in crate::app) fn apply_selected_managed_config_and_restart(&mut self) {
        let Some(node) =
            selected_node_or_notice(self, "Select a node before applying and restarting")
        else {
            return;
        };

        let plugins = plugin_states_for(self, &node);
        let path = self.managed_config_path(&node);
        let export = match ConfigExporter::write_node_config_to_path(&path, &node, &plugins) {
            Ok(export) => export,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };

        if !node_requires_restart(&node) {
            record_managed_config_applied(self, &node, &export, false);
            return;
        }

        self.restart_after_managed_config_apply(node);
    }

    fn restart_after_managed_config_apply(&mut self, node: NodeConfig) {
        let plan = self.launch_plan_for(&node);
        let log_path = self.node_log_path(&node);
        match self.supervisor.restart(&node, &plan, &log_path) {
            Ok(start) => self.record_managed_config_restart_success(&node, start),
            Err(error) => self.record_managed_config_restart_failure(&node, error),
        }
    }

    fn record_managed_config_restart_success(&mut self, node: &NodeConfig, start: ProcessStart) {
        if let Err(error) =
            self.repository
                .update_node_status(&node.id, NodeStatus::Running, Some(start.pid))
        {
            self.notice = Some(error.to_string());
            return;
        }
        self.watchdog.clear(&node.id);
        let message = format!(
            "Managed config applied and {} restarted with PID {}; log {}",
            node.name,
            start.pid,
            short_path(&start.log_path, 42)
        );
        self.record_node_event(
            node,
            EventKind::ConfigApplied,
            EventSeverity::Info,
            message.clone(),
        );
        self.notice = Some(message);
        self.reload_nodes();
    }

    fn record_managed_config_restart_failure(&mut self, node: &NodeConfig, error: anyhow::Error) {
        let _ = self
            .repository
            .update_node_status(&node.id, NodeStatus::Error, None);
        let message = format!(
            "Managed config applied to {} but restart failed: {}",
            node.name, error
        );
        self.record_node_event(
            node,
            EventKind::NodeStartFailed,
            EventSeverity::Critical,
            message.clone(),
        );
        self.notice = Some(message);
        self.reload_nodes();
    }
}
