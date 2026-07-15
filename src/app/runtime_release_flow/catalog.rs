use super::*;
use crate::app::domain::RuntimeCatalogLoad;

impl NeoNexusApp {
    pub(in crate::app) fn load_runtime_release_catalog(&mut self) {
        let request = RuntimeCatalogLoadRequest {
            source: self.runtime_catalog_source.trim().to_string(),
            signature_source: optional_text(&self.runtime_catalog_signature_source),
            ed25519_public_key: optional_text(&self.runtime_catalog_public_key),
            max_bytes: RuntimePackageManager::DEFAULT_CATALOG_MAX_BYTES,
        };

        let selected_profile_id = self.selected_runtime_catalog_profile.clone();
        match RuntimePackageManager::load_release_catalog(&request) {
            Ok(load) => self.apply_loaded_runtime_catalog(load, request, selected_profile_id),
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    fn apply_loaded_runtime_catalog(
        &mut self,
        load: RuntimeCatalogLoad,
        request: RuntimeCatalogLoadRequest,
        selected_profile_id: Option<String>,
    ) {
        if let Some(profile_id) = &selected_profile_id {
            if let Err(error) = self
                .repository
                .mark_runtime_catalog_profile_loaded(profile_id, &load)
            {
                self.session.notice = Some(error.to_string());
                return;
            }
        }

        let catalog = load.catalog;
        let count = catalog.releases.len();
        let platform = RuntimePlatform::current();
        let preferred_runtime = self
            .selected_node()
            .map_or(NodeType::NeoRs, |node| node.node_type);
        self.selected_runtime_release = catalog
            .latest_for(preferred_runtime, &platform)
            .or_else(|| catalog.compatible_releases(&platform).first().copied())
            .or_else(|| catalog.releases.first())
            .map(|release| release.id.clone());
        self.runtime_catalog_page = 0;
        self.runtime_catalog_signature_verified = load.signature_verified;
        self.runtime_catalog_bytes = load.bytes;
        self.runtime_catalog = Some(catalog);

        let trusted_signer =
            self.mark_runtime_signer_used_by_key(request.ed25519_public_key.as_deref());
        let signer_suffix = trusted_signer
            .as_ref()
            .map_or(String::new(), |label| format!("; signer {label}"));
        self.session.notice = Some(format!(
            "Runtime catalog loaded: {count} releases ({}){}",
            format_bytes(load.bytes),
            signer_suffix
        ));

        if let Some(profile_id) = selected_profile_id {
            self.reload_runtime_catalog_profiles();
            self.selected_runtime_catalog_profile = Some(profile_id);
        }
    }
}
