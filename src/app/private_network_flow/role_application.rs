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

        // Refuse a duty the node's client cannot perform. Accepting it would
        // write plugin rows and print success for something that will never
        // run.
        let availability = role_availability(node.node_type, self.selected_role);
        if let Some(reason) = availability.reason() {
            self.session.notice = Some(format!(
                "{} cannot perform the {} duty: {reason}",
                node.node_type, self.selected_role
            ));
            return;
        }

        // The duty has to outlive this session: it is what decides which
        // service sections the node's generated configuration carries, and a
        // role held only in UI state never reaches the generator.
        if let Err(error) = self
            .repository
            .set_node_role(&node.id, Some(self.selected_role))
        {
            self.session.notice = Some(error.to_string());
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
                "{} role recorded for {}; it is carried by the generated configuration",
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
