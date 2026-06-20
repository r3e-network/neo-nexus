use anyhow::Context;

use crate::runtime::{RuntimeCatalogUpgradePlan, RuntimePackageManager, RuntimePlatform};

use super::super::NeoNexusApp;
use super::model::RuntimeUpgradePolicySummary;

impl NeoNexusApp {
    pub(super) fn execute_runtime_upgrade_policy(
        &mut self,
    ) -> anyhow::Result<RuntimeUpgradePolicySummary> {
        let policy = self.runtime_upgrade_policy.clone();
        let context = self.load_runtime_upgrade_catalog(&policy)?;
        let fleet_plan = RuntimePackageManager::plan_catalog_fleet_upgrades(
            &self.nodes,
            &context.catalog,
            &RuntimePlatform::current(),
        );
        let available = fleet_plan.candidates.len();
        let candidates = fleet_plan
            .candidates
            .into_iter()
            .take(policy.max_nodes_per_run)
            .collect::<Vec<_>>();
        let limited = available > candidates.len();
        let catalog_label = context.profile.label.clone();
        let blocked_running = fleet_plan.blocked_running;
        let current_or_unavailable = fleet_plan.current_or_unavailable;

        self.publish_runtime_upgrade_catalog(context);

        if candidates.is_empty() {
            return Ok(RuntimeUpgradePolicySummary::new(
                0,
                available,
                blocked_running,
                current_or_unavailable,
                catalog_label,
                false,
            ));
        }

        let upgraded = self.apply_runtime_upgrade_candidates(
            candidates,
            &catalog_label,
            policy.max_nodes_per_run,
        )?;
        self.reload_nodes();
        Ok(RuntimeUpgradePolicySummary::new(
            upgraded,
            available,
            blocked_running,
            current_or_unavailable,
            catalog_label,
            limited,
        ))
    }

    fn apply_runtime_upgrade_candidates(
        &mut self,
        candidates: Vec<RuntimeCatalogUpgradePlan>,
        catalog_label: &str,
        limit: usize,
    ) -> anyhow::Result<usize> {
        let mut upgraded = 0usize;
        for plan in candidates {
            let Some(node) = self
                .nodes
                .iter()
                .find(|node| node.id == plan.node_id)
                .cloned()
            else {
                continue;
            };
            self.ensure_catalog_release_installed(&plan.release)
                .and_then(|installation| {
                    self.apply_runtime_installation_to_node(
                        &node,
                        &installation,
                        &plan.from_version,
                    )
                })
                .with_context(|| {
                    format!(
                        "stopped after {upgraded} of {limit} planned upgrades for {catalog_label}"
                    )
                })?;
            upgraded += 1;
        }

        Ok(upgraded)
    }
}
