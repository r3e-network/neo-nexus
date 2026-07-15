use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn reload_runtime_signer_profiles(&mut self) {
        let should_prefill = self.runtime_signer_public_key.trim().is_empty();
        match self.repository.list_runtime_signer_profiles() {
            Ok(profiles) => {
                self.runtime_signer_profiles = profiles;
                self.ensure_valid_runtime_signer_profile_selection();
                if should_prefill {
                    if let Some(profile) = self.selected_runtime_signer_profile() {
                        self.apply_runtime_signer_profile_to_form(&profile);
                    }
                }
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn ensure_valid_runtime_signer_profile_selection(&mut self) {
        let selected_exists = self
            .selected_runtime_signer_profile
            .as_ref()
            .is_some_and(|id| {
                self.runtime_signer_profiles
                    .iter()
                    .any(|profile| &profile.id == id)
            });
        if !selected_exists {
            self.selected_runtime_signer_profile = self
                .runtime_signer_profiles
                .iter()
                .find(|profile| profile.enabled)
                .or_else(|| self.runtime_signer_profiles.first())
                .map(|profile| profile.id.clone());
        }
    }

    pub(in crate::app) fn selected_runtime_signer_profile(&self) -> Option<RuntimeSignerProfile> {
        let selected_id = self.selected_runtime_signer_profile.as_deref()?;
        self.runtime_signer_profiles
            .iter()
            .find(|profile| profile.id == selected_id)
            .cloned()
    }

    pub(in crate::app) fn apply_runtime_signer_profile_to_form(
        &mut self,
        profile: &RuntimeSignerProfile,
    ) {
        self.runtime_signer_profile_label = profile.label.clone();
        self.runtime_signer_public_key = profile.ed25519_public_key.clone();
    }
}
