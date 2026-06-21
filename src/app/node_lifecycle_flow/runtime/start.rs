use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn start_node_with_mode(&mut self, index: usize, mode: StartMode) {
        let Some(node) = self.nodes.get(index).cloned() else {
            return;
        };
        let plan = self.launch_plan_for(&node);
        let plugins = self
            .repository
            .list_plugin_states(&node.id)
            .unwrap_or_default();
        let readiness = evaluate_launch_readiness(
            &node,
            &self.nodes,
            &plugins,
            self.managed_config_path(&node),
            self.node_work_dir(&node),
        );
        if let Some(blocker) = readiness.blocking_summary() {
            let message = format!("Start readiness blocked: {blocker}");
            self.fail_node_start(&node, mode, message);
            return;
        }

        if let Some(path) = plan.managed_config_path.as_ref() {
            if let Err(error) = ConfigExporter::write_node_config_to_path(path, &node, &plugins) {
                self.fail_node_start(&node, mode, error.to_string());
                return;
            }
        }

        let log_path = self.node_log_path(&node);
        match self.supervisor.start(&node, &plan, &log_path) {
            Ok(start) => {
                if let Err(error) = self.repository.update_node_status(
                    &node.id,
                    NodeStatus::Running,
                    Some(start.pid),
                ) {
                    self.notice = Some(error.to_string());
                } else {
                    match mode {
                        StartMode::Manual => {
                            self.watchdog.clear(&node.id);
                            self.notice = Some(format!(
                                "{} started; log {}",
                                node.name,
                                short_path(&start.log_path, 42)
                            ));
                            self.record_node_event(
                                &node,
                                EventKind::NodeStarted,
                                EventSeverity::Info,
                                format!("{} started with PID {}", node.name, start.pid),
                            );
                        }
                        StartMode::Watchdog { attempt } => {
                            self.notice = Some(format!(
                                "{} restarted by watchdog attempt {}; log {}",
                                node.name,
                                attempt,
                                short_path(&start.log_path, 42)
                            ));
                            self.record_node_event(
                                &node,
                                EventKind::WatchdogRestarted,
                                EventSeverity::Warning,
                                format!("{} restarted by watchdog attempt {attempt}", node.name),
                            );
                        }
                    }
                }
                self.reload_nodes();
            }
            Err(error) => self.fail_node_start(&node, mode, error.to_string()),
        }
    }
}
