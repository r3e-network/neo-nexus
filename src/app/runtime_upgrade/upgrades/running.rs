use crate::{
    diagnostics::evaluate_restart_readiness, runtime::RuntimeCatalogUpgradePlan, types::NodeConfig,
};

use super::super::super::NeoNexusApp;

impl NeoNexusApp {
    pub(in crate::app) fn upgrade_running_node_from_catalog(
        &mut self,
        node: &NodeConfig,
        plan: &RuntimeCatalogUpgradePlan,
    ) -> anyhow::Result<String> {
        let installation = self.ensure_catalog_release_installed(&plan.release)?;
        let candidate = NodeConfig {
            binary_path: installation.binary_path.clone(),
            runtime_version: installation.version.clone(),
            ..node.clone()
        };
        let plugins = self
            .repository
            .list_plugin_states(&node.id)
            .unwrap_or_default();
        let readiness = evaluate_restart_readiness(
            &candidate,
            &self.nodes,
            &plugins,
            self.managed_config_path(&candidate),
            self.node_work_dir(&candidate),
        );
        if let Some(blocker) = readiness.blocking_summary() {
            anyhow::bail!("Runtime upgrade restart readiness blocked: {blocker}");
        }

        let upgrade_message =
            self.apply_runtime_installation_to_node(node, &installation, &plan.from_version)?;
        self.reload_nodes();
        let index = self
            .nodes
            .iter()
            .position(|candidate| candidate.id == node.id)
            .ok_or_else(|| anyhow::anyhow!("upgraded node {} was not found", node.name))?;
        self.restart_node(index);
        let restart_message = self
            .notice
            .clone()
            .unwrap_or_else(|| format!("{} restarted", node.name));
        let upgraded = self
            .nodes
            .iter()
            .find(|candidate| candidate.id == node.id)
            .ok_or_else(|| anyhow::anyhow!("restarted node {} was not found", node.name))?;
        if upgraded.status.is_running()
            && upgraded.runtime_version == installation.version
            && upgraded.binary_path == installation.binary_path
        {
            Ok(format!("{upgrade_message}; {restart_message}"))
        } else {
            anyhow::bail!("{restart_message}");
        }
    }
}
