use super::super::super::NeoNexusApp;

impl NeoNexusApp {
    pub(in crate::app) fn upgrade_fleet_nodes_from_catalog(&mut self) {
        let Some(fleet_plan) = self.catalog_fleet_upgrade_plan() else {
            self.notice = Some("Load a runtime catalog before running fleet upgrades".to_string());
            return;
        };
        let candidates = fleet_plan.into_ready_candidates();
        if candidates.is_empty() {
            self.notice = Some("No fleet nodes have newer compatible catalog releases".to_string());
            return;
        }

        let mut upgraded = 0usize;
        let mut last_message = String::new();
        for plan in candidates {
            let Some(node) = self
                .nodes
                .iter()
                .find(|node| node.id == plan.node_id)
                .cloned()
            else {
                continue;
            };
            match self.apply_catalog_upgrade_plan_to_node(&node, &plan) {
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
            format!("Fleet catalog upgrade applied to {upgraded} nodes")
        });
    }
}
