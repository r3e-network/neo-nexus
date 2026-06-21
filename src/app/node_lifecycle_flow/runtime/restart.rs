use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn restart_node(&mut self, index: usize) {
        let Some(node) = self.nodes.get(index).cloned() else {
            return;
        };
        if !node.status.is_running() {
            self.notice = Some(format!("{} must be running before restart", node.name));
            return;
        }

        let plan = self.launch_plan_for(&node);
        let plugins = self
            .repository
            .list_plugin_states(&node.id)
            .unwrap_or_default();
        let readiness = evaluate_restart_readiness(
            &node,
            &self.nodes,
            &plugins,
            self.managed_config_path(&node),
            self.node_work_dir(&node),
        );
        if let Some(blocker) = readiness.blocking_summary() {
            let message = format!("Restart readiness blocked: {blocker}");
            self.record_node_start_failure(&node, message);
            return;
        }

        if let Some(path) = plan.managed_config_path.as_ref() {
            if let Err(error) = ConfigExporter::write_node_config_to_path(path, &node, &plugins) {
                self.record_node_start_failure(&node, error.to_string());
                return;
            }
        }

        let log_path = self.node_log_path(&node);
        match self.supervisor.restart(&node, &plan, &log_path) {
            Ok(start) => {
                if let Err(error) = self.repository.update_node_status(
                    &node.id,
                    NodeStatus::Running,
                    Some(start.pid),
                ) {
                    self.notice = Some(error.to_string());
                    return;
                }
                self.watchdog.clear(&node.id);
                let message = format!(
                    "{} restarted with PID {}; log {}",
                    node.name,
                    start.pid,
                    short_path(&start.log_path, 42)
                );
                self.record_node_event(
                    &node,
                    EventKind::NodeRestarted,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.notice = Some(message);
                self.reload_nodes();
            }
            Err(error) => {
                let _ = self
                    .repository
                    .update_node_status(&node.id, NodeStatus::Error, None);
                let message = format!("{} restart failed: {error}", node.name);
                self.record_node_start_failure(&node, message);
                self.reload_nodes();
            }
        }
    }
}
