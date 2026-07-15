use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn import_neo_wallet_profile_from_form(&mut self) {
        let source = self.wallet_profile_source.trim();
        if source.is_empty() {
            self.session.notice = Some("Enter a Neo wallet path before importing".to_string());
            return;
        }
        if self.wallet_profile_id.trim().is_empty() {
            self.session.notice = Some("Enter a wallet profile ID before importing".to_string());
            return;
        }
        if self.wallet_profile_label.trim().is_empty() {
            self.session.notice = Some("Enter a wallet profile label before importing".to_string());
            return;
        }

        let validated_at_unix = match current_unix_time() {
            Ok(value) => value,
            Err(error) => {
                self.session.notice = Some(error.to_string());
                return;
            }
        };
        let profile = match NeoWalletValidator::profile_from_path(
            source,
            self.wallet_profile_id.clone(),
            self.wallet_profile_label.clone(),
            validated_at_unix,
        ) {
            Ok(profile) => profile,
            Err(error) => {
                self.session.notice = Some(error.to_string());
                return;
            }
        };

        match self.repository.upsert_neo_wallet_profile(&profile) {
            Ok(()) => {
                let selected_id = profile.id.clone();
                let message = format!("Wallet profile imported: {}", profile.label);
                self.selected_neo_wallet_profile = Some(selected_id.clone());
                self.reload_neo_wallet_profiles();
                self.selected_neo_wallet_profile = Some(selected_id.clone());
                self.record_event(
                    None,
                    None,
                    EventKind::NeoWalletProfileImported,
                    EventSeverity::Info,
                    format!(
                        "wallet-profile:{} imported from {}",
                        profile.id, profile.source_path
                    ),
                );
                self.session.notice = Some(message);
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
