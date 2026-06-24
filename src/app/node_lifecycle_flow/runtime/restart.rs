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

        // Delegate the export -> supervise -> persist-status pipeline to the
        // shared core so the GUI and a future CLI use one restart path.
        let managed_config_path = plan.managed_config_path.as_deref();
        let log_path = self.node_log_path(&node);
        let outcome = execute_node_launch(
            &self.repository,
            &mut self.supervisor,
            &node,
            &plan,
            &log_path,
            LaunchAction::Restart,
            managed_config_path.map(|path| ManagedConfig {
                path,
                plugins: &plugins,
            }),
        );
        match outcome {
            NodeLaunchOutcome::Started { pid, log_path } => {
                self.watchdog.clear(&node.id);
                let message = format!(
                    "{} restarted with PID {}; log {}",
                    node.name,
                    pid,
                    short_path(&log_path, 42)
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
            NodeLaunchOutcome::Failed { message } => {
                let message = format!("{} restart failed: {message}", node.name);
                self.record_node_start_failure(&node, message);
                self.reload_nodes();
            }
        }
    }
}
