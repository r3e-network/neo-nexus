use crate::{
    events::{EventKind, EventSeverity},
    runtime::{RuntimeCatalogUpgradePlan, RuntimeInstallation},
    types::{NodeConfig, NodeStatus},
};

use super::{super::super::NeoNexusApp, input::runtime_installation_node_input};
use crate::app::short_path;

impl NeoNexusApp {
    pub(in crate::app) fn apply_catalog_upgrade_plan_to_node(
        &mut self,
        node: &NodeConfig,
        plan: &RuntimeCatalogUpgradePlan,
    ) -> anyhow::Result<String> {
        if node.status == NodeStatus::Running {
            return self.upgrade_running_node_from_catalog(node, plan);
        }

        self.ensure_catalog_release_installed(&plan.release)
            .and_then(|installation| {
                self.apply_runtime_installation_to_node(node, &installation, &plan.from_version)
            })
    }

    pub(in crate::app) fn apply_runtime_installation_to_node(
        &mut self,
        node: &NodeConfig,
        installation: &RuntimeInstallation,
        from_version: &str,
    ) -> anyhow::Result<String> {
        let input = runtime_installation_node_input(node, installation);
        let updated = self.repository.update_node(&node.id, input)?;
        let message = format!(
            "{} upgraded from {} to {} using {}",
            updated.name,
            from_version,
            installation.version,
            short_path(&installation.binary_path, 54)
        );
        self.record_node_event(
            &updated,
            EventKind::RuntimeApplied,
            EventSeverity::Info,
            message.clone(),
        );
        Ok(message)
    }
}
