use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn apply_selected_role(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.session.notice = Some("Select a node before applying a role".to_string());
            return;
        };

        if node.status.is_active() {
            self.session.notice = Some("Stop the selected node before applying a role".to_string());
            return;
        }

        let plan = RolePlanner::plan(&node, self.selected_role);
        for change in &plan.plugin_changes {
            if let Err(error) =
                self.repository
                    .set_plugin_enabled(&node.id, change.plugin_id, change.enabled)
            {
                self.session.notice = Some(error.to_string());
                return;
            }
        }

        let message = if plan.change_count() == 0 {
            format!(
                "{} role planned for {}; runtime posture is managed by managed config",
                plan.role, node.name
            )
        } else {
            format!(
                "{} role applied to {}; {} plugin settings updated",
                plan.role,
                node.name,
                plan.change_count()
            )
        };
        self.record_node_event(
            &node,
            EventKind::RoleApplied,
            EventSeverity::Info,
            message.clone(),
        );
        self.session.notice = Some(message);
    }
}
