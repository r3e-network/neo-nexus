use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn private_network_template_source_node(&self) -> Option<&NodeConfig> {
        if let Some(node) = self
            .selected_node()
            .filter(|node| node.node_type == self.private_network_runtime)
        {
            return Some(node);
        }

        self.nodes
            .iter()
            .find(|node| node.node_type == self.private_network_runtime)
    }

    pub(in crate::app) fn materialize_private_network_plan(&mut self) {
        let plan = PrivateNetworkPlanner::plan(
            self.private_network_template,
            self.private_network_runtime,
        );
        let Some(template_node) = self.private_network_template_source_node().cloned() else {
            self.notice = Some(format!(
                "Create a {} node first so NeoNexus can reuse its binary and version",
                self.private_network_runtime
            ));
            return;
        };

        let conflicts = plan.conflicts_with(&self.nodes);
        if let Some(conflict) = conflicts.first() {
            self.notice = Some(format!(
                "Private network plan has a conflict: {}",
                conflict.detail
            ));
            return;
        }

        let inputs = match plan.to_new_nodes(&template_node) {
            Ok(inputs) => inputs,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };

        let inputs_with_plugins: Vec<_> = plan
            .nodes
            .iter()
            .zip(inputs)
            .map(|(planned, input)| {
                let role_plan =
                    RolePlanner::plan_for(input.node_type, input.storage_engine, planned.role);
                let plugins = role_plan
                    .plugin_changes
                    .into_iter()
                    .map(|change| PluginState {
                        plugin_id: change.plugin_id,
                        enabled: change.enabled,
                    })
                    .collect();
                (input, plugins)
            })
            .collect();

        match self
            .repository
            .create_nodes_with_plugins(inputs_with_plugins)
        {
            Ok(created) => {
                if let Some(first) = created.first() {
                    self.selected_node = Some(first.id.clone());
                }
                self.private_network_last_export_root = None;
                self.private_network_last_validation = None;
                let message = format!(
                    "{} materialized as {} {} private node definitions from {}",
                    plan.template,
                    created.len(),
                    plan.node_type,
                    template_node.name
                );
                self.record_event_notice(
                    EventKind::PrivateNetworkMaterialized,
                    EventSeverity::Info,
                    message,
                );
                self.node_page = 0;
                self.reload_nodes();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn private_network_materialized_nodes(
        &self,
        plan: &PrivateNetworkPlan,
    ) -> anyhow::Result<Vec<NodeConfig>> {
        let mut nodes = Vec::with_capacity(plan.nodes.len());
        for planned in &plan.nodes {
            let node = self
                .nodes
                .iter()
                .find(|node| {
                    node.name == planned.name
                        && node.node_type == planned.node_type
                        && node.network == planned.network
                })
                .cloned()
                .with_context(|| {
                    format!("Create {} before exporting a launch pack", planned.name)
                })?;
            nodes.push(node);
        }
        Ok(nodes)
    }
}
