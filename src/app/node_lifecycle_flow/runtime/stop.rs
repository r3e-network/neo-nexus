use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn stop_node(&mut self, index: usize) {
        let Some(node) = self.fleet.nodes.get(index).cloned() else {
            return;
        };

        match self.supervisor.stop(&node.id) {
            Ok(stop) => {
                self.watchdog.clear(&node.id);
                if let Err(error) =
                    self.repository
                        .update_node_status(&node.id, NodeStatus::Stopped, None)
                {
                    self.session.notice = Some(error.to_string());
                } else {
                    let detail = stop.map_or_else(
                        || "no supervised process was active".to_string(),
                        |stop| stop.operator_summary(),
                    );
                    let message = format!("{} stopped ({detail})", node.name);
                    self.session.notice = Some(message.clone());
                    self.record_node_event(
                        &node,
                        EventKind::NodeStopped,
                        EventSeverity::Info,
                        message,
                    );
                }
                self.reload_nodes();
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
