use std::{cmp::Ordering, path::PathBuf};

use crate::types::{NodeConfig, NodeStatus};

use super::super::{compare_versions, RuntimeInstallation, RuntimePlatform, RuntimeReleaseCatalog};
use super::RuntimePackageManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeUpgradePlan {
    pub node_id: String,
    pub node_name: String,
    pub package_id: String,
    pub from_version: String,
    pub to_version: String,
    pub from_binary_path: PathBuf,
    pub to_binary_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCatalogUpgradePlan {
    pub node_id: String,
    pub node_name: String,
    pub from_version: String,
    pub to_version: String,
    pub release: super::super::RuntimeRelease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCatalogFleetPlan {
    pub stopped_candidates: Vec<RuntimeCatalogUpgradePlan>,
    pub running_candidates: Vec<RuntimeCatalogUpgradePlan>,
    pub blocked_active: usize,
    pub current_or_unavailable: usize,
}

impl RuntimeCatalogFleetPlan {
    pub fn ready_count(&self) -> usize {
        self.stopped_candidates.len() + self.running_candidates.len()
    }

    pub fn stopped_ready_count(&self) -> usize {
        self.stopped_candidates.len()
    }

    pub fn running_ready_count(&self) -> usize {
        self.running_candidates.len()
    }

    pub fn ready_breakdown_label(&self) -> String {
        format!(
            "{} ready ({} stopped, {} running)",
            self.ready_count(),
            self.stopped_ready_count(),
            self.running_ready_count()
        )
    }

    pub fn into_ready_candidates(self) -> Vec<RuntimeCatalogUpgradePlan> {
        self.stopped_candidates
            .into_iter()
            .chain(self.running_candidates)
            .collect()
    }
}

impl RuntimePackageManager {
    pub fn plan_node_upgrade(
        node: &NodeConfig,
        installations: &[RuntimeInstallation],
        platform: &RuntimePlatform,
    ) -> Option<RuntimeUpgradePlan> {
        let best = installations
            .iter()
            .filter(|installation| {
                installation.node_type == node.node_type && &installation.platform == platform
            })
            .max_by(|left, right| compare_installations(left, right))?;

        if node.runtime_version == best.version && node.binary_path == best.binary_path {
            return None;
        }

        Some(RuntimeUpgradePlan {
            node_id: node.id.clone(),
            node_name: node.name.clone(),
            package_id: best.package_id.clone(),
            from_version: node.runtime_version.clone(),
            to_version: best.version.clone(),
            from_binary_path: node.binary_path.clone(),
            to_binary_path: best.binary_path.clone(),
        })
    }

    pub fn plan_catalog_upgrade(
        node: &NodeConfig,
        catalog: &RuntimeReleaseCatalog,
        platform: &RuntimePlatform,
    ) -> Option<RuntimeCatalogUpgradePlan> {
        let release = catalog.latest_for(node.node_type, platform)?;
        if compare_versions(&release.version, &node.runtime_version) != Ordering::Greater {
            return None;
        }

        Some(RuntimeCatalogUpgradePlan {
            node_id: node.id.clone(),
            node_name: node.name.clone(),
            from_version: node.runtime_version.clone(),
            to_version: release.version.clone(),
            release: release.clone(),
        })
    }

    pub fn plan_catalog_fleet_upgrades(
        nodes: &[NodeConfig],
        catalog: &RuntimeReleaseCatalog,
        platform: &RuntimePlatform,
    ) -> RuntimeCatalogFleetPlan {
        let mut stopped_candidates = Vec::new();
        let mut running_candidates = Vec::new();
        let mut blocked_active = 0usize;
        let mut current_or_unavailable = 0usize;

        for node in nodes {
            match Self::plan_catalog_upgrade(node, catalog, platform) {
                Some(plan) if node.status == NodeStatus::Stopped => stopped_candidates.push(plan),
                Some(plan) if node.status == NodeStatus::Running => running_candidates.push(plan),
                Some(_) => blocked_active += 1,
                None => current_or_unavailable += 1,
            }
        }

        RuntimeCatalogFleetPlan {
            stopped_candidates,
            running_candidates,
            blocked_active,
            current_or_unavailable,
        }
    }
}

fn compare_installations(left: &RuntimeInstallation, right: &RuntimeInstallation) -> Ordering {
    compare_versions(&left.version, &right.version)
        .then_with(|| left.installed_at_unix.cmp(&right.installed_at_unix))
        .then_with(|| left.package_id.cmp(&right.package_id))
}
