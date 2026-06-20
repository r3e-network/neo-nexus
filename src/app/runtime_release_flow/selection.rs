use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn ensure_valid_runtime_release_selection(&mut self) {
        let Some(catalog) = &self.runtime_catalog else {
            self.selected_runtime_release = None;
            self.runtime_catalog_page = 0;
            return;
        };
        let platform = RuntimePlatform::current();
        let releases = catalog.releases.clone();
        let visible = self.filtered_runtime_releases(&releases, &platform);

        let selected_exists = self
            .selected_runtime_release
            .as_ref()
            .is_some_and(|id| visible.iter().any(|release| &release.id == id));
        if !selected_exists {
            self.selected_runtime_release = visible
                .iter()
                .find(|release| release.platform_matches(&platform))
                .map(|release| release.id.clone())
                .or_else(|| visible.first().map(|release| release.id.clone()));
            self.runtime_catalog_page = 0;
        }

        self.runtime_catalog_page = clamp_page(
            self.runtime_catalog_page,
            visible.len(),
            RUNTIME_CATALOG_PAGE_SIZE,
        );
    }
}
