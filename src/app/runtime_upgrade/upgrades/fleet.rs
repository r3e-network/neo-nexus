use super::super::super::NeoNexusApp;

impl NeoNexusApp {
    pub(in crate::app) fn upgrade_stopped_nodes_from_catalog(&mut self) {
        let Some(fleet_plan) = self.catalog_fleet_upgrade_plan() else {
            self.notice = Some("Load a runtime catalog before running fleet upgrades".to_string());
            return;
        };
        if fleet_plan.candidates.is_empty() {
            self.notice =
                Some("No stopped nodes have newer compatible catalog releases".to_string());
            return;
        }

        let mut upgraded = 0usize;
        let mut last_message = String::new();
        for plan in fleet_plan.candidates {
            let Some(node) = self
                .nodes
                .iter()
                .find(|node| node.id == plan.node_id)
                .cloned()
            else {
                continue;
            };
            match self
                .ensure_catalog_release_installed(&plan.release)
                .and_then(|installation| {
                    self.apply_runtime_installation_to_node(
                        &node,
                        &installation,
                        &plan.from_version,
                    )
                }) {
                Ok(message) => {
                    upgraded += 1;
                    last_message = message;
                }
                Err(error) => {
                    self.notice = Some(format!(
                        "Fleet catalog upgrade stopped after {upgraded} nodes: {error}"
                    ));
                    self.reload_nodes();
                    return;
                }
            }
        }

        self.reload_nodes();
        self.notice = Some(if upgraded == 1 {
            last_message
        } else {
            format!("Fleet catalog upgrade applied to {upgraded} stopped nodes")
        });
    }
}
