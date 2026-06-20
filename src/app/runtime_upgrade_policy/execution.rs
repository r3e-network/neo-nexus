use anyhow::Context;

use crate::runtime::{RuntimeCatalogUpgradePlan, RuntimePackageManager, RuntimePlatform};

use super::super::NeoNexusApp;
use super::model::{RuntimeUpgradePolicyBreakdown, RuntimeUpgradePolicySummary};

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
        let available = fleet_plan.ready_count();
        let stopped_ready = fleet_plan.stopped_ready_count();
        let running_ready = fleet_plan.running_ready_count();
        let planned_stopped = stopped_ready.min(policy.max_nodes_per_run);
        let planned_running = running_ready.min(policy.max_nodes_per_run - planned_stopped);
        let full_breakdown = RuntimeUpgradePolicyBreakdown {
            stopped_ready,
            running_ready,
            planned_stopped,
            planned_running,
            blocked_active: fleet_plan.blocked_active,
            current_or_unavailable: fleet_plan.current_or_unavailable,
        };
        let candidates = fleet_plan
            .into_ready_candidates()
            .into_iter()
            .take(policy.max_nodes_per_run)
            .collect::<Vec<_>>();
        let limited = available > candidates.len();
        let catalog_label = context.profile.label.clone();

        self.publish_runtime_upgrade_catalog(context);

        if candidates.is_empty() {
            return Ok(RuntimeUpgradePolicySummary::new(
                0,
                RuntimeUpgradePolicyBreakdown {
                    planned_stopped: 0,
                    planned_running: 0,
                    ..full_breakdown
                },
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
            full_breakdown,
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
            self.apply_catalog_upgrade_plan_to_node(&node, &plan)
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
