use crate::app::{
    domain::{
        format_bytes, EventKind, EventSeverity, RuntimeInstallation, RuntimePackageManager,
        RuntimePlatform, RuntimeRelease,
    },
    short_path, NeoNexusApp,
};

impl NeoNexusApp {
    fn installed_catalog_release(&self, release: &RuntimeRelease) -> Option<RuntimeInstallation> {
        let platform = RuntimePlatform::current();
        self.runtime_installations()
            .into_iter()
            .find(|installation| {
                installation.package_id == release.id
                    && installation.node_type == release.node_type
                    && installation.version == release.version
                    && installation.platform == platform
            })
    }

    pub(in crate::app) fn ensure_catalog_release_installed(
        &mut self,
        release: &RuntimeRelease,
    ) -> anyhow::Result<RuntimeInstallation> {
        if let Some(installation) = self.installed_catalog_release(release) {
            return Ok(installation);
        }

        let download = RuntimePackageManager::download_https(
            &release.download_request(),
            self.runtime_download_dir(),
        )?;
        self.record_event(
            None,
            None,
            EventKind::RuntimeDownloaded,
            EventSeverity::Info,
            format!(
                "Catalog upgrade downloaded {} {} from {} ({})",
                release.node_type,
                release.version,
                short_path(&download.path, 54),
                format_bytes(download.bytes)
            ),
        );

        let manifest = release.manifest_for_source(download.path.clone());
        let installation = RuntimePackageManager::install(&manifest, self.runtime_install_root())?;
        self.repository.upsert_runtime_installation(&installation)?;
        self.selected_runtime_installation = Some(installation.package_id.clone());
        self.record_event(
            None,
            None,
            EventKind::RuntimeInstalled,
            EventSeverity::Info,
            format!(
                "Catalog upgrade installed {} {} at {}",
                installation.node_type,
                installation.version,
                short_path(&installation.binary_path, 54)
            ),
        );
        Ok(installation)
    }
}
