use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn reload_runtime_catalog_profiles(&mut self) {
        let should_prefill = self.runtime_catalog_source.trim().is_empty();
        match self.repository.list_runtime_catalog_profiles() {
            Ok(profiles) => {
                self.runtime_catalog_profiles = profiles;
                self.ensure_valid_runtime_catalog_profile_selection();
                if should_prefill {
                    if let Some(profile) = self.selected_runtime_catalog_profile() {
                        self.apply_runtime_catalog_profile_to_form(&profile);
                    }
                }
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn ensure_valid_runtime_catalog_profile_selection(&mut self) {
        let selected_exists = self
            .selected_runtime_catalog_profile
            .as_ref()
            .is_some_and(|id| {
                self.runtime_catalog_profiles
                    .iter()
                    .any(|profile| &profile.id == id)
            });
        if !selected_exists {
            self.selected_runtime_catalog_profile = self
                .runtime_catalog_profiles
                .first()
                .map(|profile| profile.id.clone());
        }
    }

    pub(in crate::app) fn selected_runtime_catalog_profile(&self) -> Option<RuntimeCatalogProfile> {
        let selected_id = self.selected_runtime_catalog_profile.as_deref()?;
        self.runtime_catalog_profiles
            .iter()
            .find(|profile| profile.id == selected_id)
            .cloned()
    }

    pub(in crate::app) fn apply_runtime_catalog_profile_to_form(
        &mut self,
        profile: &RuntimeCatalogProfile,
    ) {
        self.runtime_catalog_profile_label = profile.label.clone();
        self.runtime_catalog_source = profile.source.clone();
        self.runtime_catalog_signature_source =
            profile.signature_source.clone().unwrap_or_default();
        self.runtime_catalog_public_key = profile.ed25519_public_key.clone().unwrap_or_default();
    }

    pub(in crate::app) fn load_selected_runtime_catalog_profile_into_form(&mut self) {
        let Some(profile) = self.selected_runtime_catalog_profile() else {
            self.session.notice = Some("Select a runtime catalog profile first".to_string());
            return;
        };
        self.apply_runtime_catalog_profile_to_form(&profile);
        self.session.notice = Some(format!("Runtime catalog profile loaded: {}", profile.label));
    }

    pub(in crate::app) fn save_runtime_catalog_profile(&mut self) {
        let existing = self.selected_runtime_catalog_profile();
        let id = existing.as_ref().map_or_else(
            || format!("runtime-catalog-{}", Uuid::new_v4()),
            |profile| profile.id.clone(),
        );
        let fallback_label = existing
            .as_ref()
            .map_or("Runtime catalog", |profile| profile.label.as_str());
        let profile = RuntimeCatalogProfile {
            id: id.clone(),
            label: non_empty_text(&self.runtime_catalog_profile_label, fallback_label),
            source: self.runtime_catalog_source.trim().to_string(),
            signature_source: optional_text(&self.runtime_catalog_signature_source),
            ed25519_public_key: optional_text(&self.runtime_catalog_public_key),
            max_bytes: RuntimePackageManager::DEFAULT_CATALOG_MAX_BYTES,
            enabled: existing.as_ref().is_none_or(|profile| profile.enabled),
            last_loaded_at_unix: existing
                .as_ref()
                .and_then(|profile| profile.last_loaded_at_unix),
            last_signature_verified: existing
                .as_ref()
                .and_then(|profile| profile.last_signature_verified),
            last_bytes: existing.as_ref().and_then(|profile| profile.last_bytes),
        };

        match self.repository.upsert_runtime_catalog_profile(&profile) {
            Ok(()) => {
                self.selected_runtime_catalog_profile = Some(id);
                self.reload_runtime_catalog_profiles();
                self.session.notice = Some(format!("Runtime catalog profile saved: {}", profile.label));
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn delete_selected_runtime_catalog_profile(&mut self) {
        let Some(profile) = self.selected_runtime_catalog_profile() else {
            self.session.notice = Some("Select a runtime catalog profile first".to_string());
            return;
        };
        match self.repository.delete_runtime_catalog_profile(&profile.id) {
            Ok(()) => {
                self.selected_runtime_catalog_profile = None;
                self.reload_runtime_catalog_profiles();
                self.session.notice = Some(format!(
                    "Runtime catalog profile deleted: {}",
                    profile.label
                ));
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
