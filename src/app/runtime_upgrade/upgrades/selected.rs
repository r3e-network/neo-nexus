use super::super::super::NeoNexusApp;

impl NeoNexusApp {
    pub(in crate::app) fn upgrade_selected_node_from_catalog(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.session.notice = Some("Select a node before running a catalog upgrade".to_string());
            return;
        };
        if node.status.is_starting() {
            self.session.notice = Some(
                "Wait for the selected node to finish starting before running a catalog upgrade"
                    .to_string(),
            );
            return;
        }

        let Some(plan) = self.catalog_upgrade_plan_for_node(&node) else {
            self.session.notice =
                Some("No newer compatible catalog release for the selected node".to_string());
            return;
        };

        let result = if node.status.is_running() {
            self.upgrade_running_node_from_catalog(&node, &plan)
        } else {
            self.ensure_catalog_release_installed(&plan.release)
                .and_then(|installation| {
                    self.apply_runtime_installation_to_node(
                        &node,
                        &installation,
                        &plan.from_version,
                    )
                })
        };

        match result {
            Ok(message) => {
                self.session.notice = Some(message);
                self.reload_nodes();
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
