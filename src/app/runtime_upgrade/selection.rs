use crate::app::domain::{
    filter_runtime_installations, filter_runtime_releases, RuntimeCatalogFleetPlan,
    RuntimeCatalogUpgradePlan, RuntimeInstallation, RuntimeInstallationFilter,
    RuntimePackageManager, RuntimePlatform, RuntimeRelease, RuntimeReleaseFilter,
};

use super::super::{clamp_page, NeoNexusApp, RUNTIME_PAGE_SIZE};

impl NeoNexusApp {
    pub(in crate::app) fn selected_runtime_release(&self) -> Option<RuntimeRelease> {
        let selected_id = self.selected_runtime_release.as_deref()?;
        self.runtime_catalog.as_ref()?.get(selected_id).cloned()
    }

    pub(in crate::app) fn runtime_installation_filter(&self) -> RuntimeInstallationFilter {
        RuntimeInstallationFilter::new(
            self.runtime_inventory_type_filter,
            self.runtime_inventory_signed_filter,
            self.runtime_inventory_platform_filter,
            self.runtime_inventory_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_runtime_installations(
        &self,
        installations: &[RuntimeInstallation],
    ) -> Vec<RuntimeInstallation> {
        filter_runtime_installations(
            installations,
            &RuntimePlatform::current(),
            &self.runtime_installation_filter(),
        )
    }

    pub(in crate::app) fn runtime_release_filter(&self) -> RuntimeReleaseFilter {
        RuntimeReleaseFilter::new(
            self.runtime_catalog_type_filter,
            self.runtime_catalog_platform_filter,
            self.runtime_catalog_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_runtime_releases(
        &self,
        releases: &[RuntimeRelease],
        platform: &RuntimePlatform,
    ) -> Vec<RuntimeRelease> {
        filter_runtime_releases(releases, platform, &self.runtime_release_filter())
    }

    pub(in crate::app) fn catalog_upgrade_plan_for_node(
        &self,
        node: &crate::app::domain::NodeConfig,
    ) -> Option<RuntimeCatalogUpgradePlan> {
        RuntimePackageManager::plan_catalog_upgrade(
            node,
            self.runtime_catalog.as_ref()?,
            &RuntimePlatform::current(),
        )
    }

    pub(in crate::app) fn catalog_fleet_upgrade_plan(&self) -> Option<RuntimeCatalogFleetPlan> {
        Some(RuntimePackageManager::plan_catalog_fleet_upgrades(
            &self.nodes,
            self.runtime_catalog.as_ref()?,
            &RuntimePlatform::current(),
        ))
    }

    pub(in crate::app) fn load_selected_runtime_release_into_draft(&mut self) {
        let Some(release) = self.selected_runtime_release() else {
            self.notice = Some("Select a runtime release first".to_string());
            return;
        };

        self.runtime_package_draft.load_release(&release);
        self.notice = Some(format!(
            "Runtime draft loaded from catalog: {} {}",
            release.node_type, release.version
        ));
    }

    pub(in crate::app) fn ensure_valid_runtime_selection(
        &mut self,
        installations: &[RuntimeInstallation],
    ) {
        let visible = self.filtered_runtime_installations(installations);
        let selected_exists = self
            .selected_runtime_installation
            .as_ref()
            .is_some_and(|id| visible.iter().any(|item| &item.package_id == id));
        if !selected_exists {
            self.selected_runtime_installation =
                visible.first().map(|item| item.package_id.clone());
            self.runtime_page = 0;
        }
        self.runtime_page = clamp_page(self.runtime_page, visible.len(), RUNTIME_PAGE_SIZE);
    }

    pub(in crate::app) fn selected_runtime_installation(
        &self,
        installations: &[RuntimeInstallation],
    ) -> Option<RuntimeInstallation> {
        let selected_id = self.selected_runtime_installation.as_deref()?;
        installations
            .iter()
            .find(|installation| installation.package_id == selected_id)
            .cloned()
    }
}
