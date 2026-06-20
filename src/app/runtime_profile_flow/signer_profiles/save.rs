use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn save_runtime_signer_profile(&mut self) {
        let existing = self.selected_runtime_signer_profile();
        let id = existing.as_ref().map_or_else(
            || format!("runtime-signer-{}", Uuid::new_v4()),
            |profile| profile.id.clone(),
        );
        let fallback_label = existing
            .as_ref()
            .map_or("Trusted runtime signer", |profile| profile.label.as_str());
        let created_at_unix = match existing
            .as_ref()
            .map_or_else(current_unix_time, |profile| Ok(profile.created_at_unix))
        {
            Ok(value) => value,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };
        let Some(public_key) = self.runtime_signer_key_candidate() else {
            self.notice =
                Some("Enter a signer public key before saving a trusted signer".to_string());
            return;
        };
        let profile = RuntimeSignerProfile {
            id: id.clone(),
            label: non_empty_text(&self.runtime_signer_profile_label, fallback_label),
            ed25519_public_key: public_key,
            enabled: existing.as_ref().is_none_or(|profile| profile.enabled),
            created_at_unix,
            last_used_at_unix: existing
                .as_ref()
                .and_then(|profile| profile.last_used_at_unix),
        };

        match self.repository.upsert_runtime_signer_profile(&profile) {
            Ok(()) => {
                self.selected_runtime_signer_profile = Some(id);
                self.runtime_signer_public_key = profile.ed25519_public_key.clone();
                self.reload_runtime_signer_profiles();
                self.notice = Some(format!("Trusted signer saved: {}", profile.label));
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    fn runtime_signer_key_candidate(&self) -> Option<String> {
        [
            self.runtime_signer_public_key.as_str(),
            self.runtime_catalog_public_key.as_str(),
            self.runtime_package_draft.ed25519_public_key.as_str(),
        ]
        .into_iter()
        .map(str::trim)
        .find(|value| !value.is_empty())
        .map(ToString::to_string)
    }
}
