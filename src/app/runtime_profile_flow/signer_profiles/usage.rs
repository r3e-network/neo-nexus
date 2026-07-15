use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn use_selected_runtime_signer_for_catalog(&mut self) {
        let Some(profile) = self.selected_runtime_signer_profile() else {
            self.session.notice = Some("Select a trusted signer first".to_string());
            return;
        };
        let Some(public_key) = profile.public_key_if_enabled().map(str::to_string) else {
            self.session.notice = Some(format!("Trusted signer is disabled: {}", profile.label));
            return;
        };

        self.apply_runtime_signer_profile_to_form(&profile);
        self.runtime_catalog_public_key = public_key;
        match self.mark_runtime_signer_profile_used(&profile.id) {
            Ok(()) => {
                self.session.notice = Some(format!(
                    "Trusted signer applied to runtime catalog: {}",
                    profile.label
                ));
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn use_selected_runtime_signer_for_package(&mut self) {
        let Some(profile) = self.selected_runtime_signer_profile() else {
            self.session.notice = Some("Select a trusted signer first".to_string());
            return;
        };
        let Some(public_key) = profile.public_key_if_enabled().map(str::to_string) else {
            self.session.notice = Some(format!("Trusted signer is disabled: {}", profile.label));
            return;
        };

        self.apply_runtime_signer_profile_to_form(&profile);
        self.runtime_package_draft.ed25519_public_key = public_key;
        match self.mark_runtime_signer_profile_used(&profile.id) {
            Ok(()) => {
                self.session.notice = Some(format!(
                    "Trusted signer applied to runtime package: {}",
                    profile.label
                ));
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn mark_runtime_signer_profile_used(
        &mut self,
        profile_id: &str,
    ) -> anyhow::Result<()> {
        self.repository
            .mark_runtime_signer_profile_used(profile_id, current_unix_time()?)?;
        self.selected_runtime_signer_profile = Some(profile_id.to_string());
        self.reload_runtime_signer_profiles();
        self.selected_runtime_signer_profile = Some(profile_id.to_string());
        Ok(())
    }

    pub(in crate::app) fn mark_runtime_signer_used_by_key(
        &mut self,
        public_key: Option<&str>,
    ) -> Option<String> {
        let public_key = public_key
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        let profile = self
            .runtime_signer_profiles
            .iter()
            .find(|profile| profile.public_key_if_enabled() == Some(public_key))
            .cloned()?;

        if self.mark_runtime_signer_profile_used(&profile.id).is_ok() {
            Some(profile.label)
        } else {
            None
        }
    }
}
