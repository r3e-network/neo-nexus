use super::*;

mod restart;
mod start;
mod stop;

impl NeoNexusApp {
    pub(in crate::app) fn start_selected_node(&mut self) {
        let Some(index) = self.selected_node_index() else {
            self.session.notice = Some("Select a node before starting it".to_string());
            return;
        };
        self.start_node(index);
    }

    pub(in crate::app) fn stop_selected_node(&mut self) {
        let Some(index) = self.selected_node_index() else {
            self.session.notice = Some("Select a node before stopping it".to_string());
            return;
        };
        self.stop_node(index);
    }

    pub(in crate::app) fn restart_selected_node(&mut self) {
        let Some(index) = self.selected_node_index() else {
            self.session.notice = Some("Select a running node before restarting it".to_string());
            return;
        };
        self.restart_node(index);
    }

    pub(in crate::app) fn start_node(&mut self, index: usize) {
        self.start_node_with_mode(index, StartMode::Manual);
    }

    /// Records a failed node start/restart as a critical event and surfaces it
    /// on the operator notice line. Shared by every manual start and restart
    /// failure path so the event kind, severity, and notice stay consistent.
    pub(in crate::app) fn record_node_start_failure(&mut self, node: &NodeConfig, message: String) {
        self.record_node_event(
            node,
            EventKind::NodeStartFailed,
            EventSeverity::Critical,
            message.clone(),
        );
        self.session.notice = Some(message);
    }

    /// Handles a start failure for either start mode: manual starts record a
    /// failure event and notice, while watchdog-driven starts defer to the
    /// watchdog restart scheduler (which sets its own notice).
    pub(in crate::app) fn fail_node_start(
        &mut self,
        node: &NodeConfig,
        mode: StartMode,
        message: String,
    ) {
        match mode {
            StartMode::Manual => self.record_node_start_failure(node, message),
            StartMode::Watchdog { .. } => self.schedule_watchdog_restart(node, &message),
        }
    }
}
