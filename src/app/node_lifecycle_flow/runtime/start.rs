use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn start_node_with_mode(&mut self, index: usize, mode: StartMode) {
        let Some(node) = self.fleet.nodes.get(index).cloned() else {
            return;
        };
        let plan = self.launch_plan_for(&node);
        let plugins = self
            .repository
            .list_plugin_states(&node.id)
            .unwrap_or_default();
        let readiness = evaluate_launch_readiness(
            &node,
            &self.fleet.nodes,
            &plugins,
            self.managed_config_path(&node),
            self.node_work_dir(&node),
        );
        if let Some(blocker) = readiness.blocking_summary() {
            let message = format!("Start readiness blocked: {blocker}");
            self.fail_node_start(&node, mode, message);
            return;
        }

        // Delegate the export -> supervise -> persist-status pipeline to the
        // shared core so the GUI and a future CLI use one launch path.
        let managed_config_path = plan.managed_config_path.as_deref();
        let log_path = self.node_log_path(&node);
        let outcome = execute_node_launch(
            &self.repository,
            &mut self.supervisor,
            &node,
            &plan,
            &log_path,
            LaunchAction::Start,
            managed_config_path.map(|path| ManagedConfig {
                path,
                plugins: &plugins,
            }),
        );
        match outcome {
            NodeLaunchOutcome::Started { pid, log_path } => {
                match mode {
                    StartMode::Manual => {
                        self.watchdog.clear(&node.id);
                        self.session.notice = Some(format!(
                            "{} started; log {}",
                            node.name,
                            short_path(&log_path, 42)
                        ));
                        self.record_node_event(
                            &node,
                            EventKind::NodeStarted,
                            EventSeverity::Info,
                            format!("{} started with PID {}", node.name, pid),
                        );
                    }
                    StartMode::Watchdog { attempt } => {
                        self.session.notice = Some(format!(
                            "{} restarted by watchdog attempt {}; log {}",
                            node.name,
                            attempt,
                            short_path(&log_path, 42)
                        ));
                        self.record_node_event(
                            &node,
                            EventKind::WatchdogRestarted,
                            EventSeverity::Warning,
                            format!("{} restarted by watchdog attempt {attempt}", node.name),
                        );
                    }
                }
                self.reload_nodes();
            }
            NodeLaunchOutcome::Failed { message } => {
                self.fail_node_start(&node, mode, message);
            }
        }
    }
}
