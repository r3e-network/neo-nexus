use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn mark_selected_neo_wallet_profile_used(&mut self) {
        let Some(profile) = self.selected_neo_wallet_profile() else {
            self.session.notice = Some("Select a wallet profile first".to_string());
            return;
        };
        match self.mark_neo_wallet_profile_used_and_reselect(&profile) {
            Ok(()) => {
                self.record_event(
                    None,
                    None,
                    EventKind::NeoWalletProfileUsed,
                    EventSeverity::Info,
                    format!(
                        "wallet-profile:{} selected for operator workflow",
                        profile.id
                    ),
                );
                self.session.notice = Some(format!("Wallet profile marked used: {}", profile.label));
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn use_selected_neo_wallet_profile_for_private_network_signer_refs(
        &mut self,
    ) {
        let Some(profile) = self.selected_neo_wallet_profile() else {
            self.session.notice = Some("Select a wallet profile first".to_string());
            return;
        };
        let Some(public_key) = profile.contract_public_keys.first().cloned() else {
            self.session.notice = Some(format!(
                "Wallet profile has no contract public key: {}",
                profile.label
            ));
            return;
        };

        let committee_keys =
            committee_keys_with_wallet_profile(&self.private_network_committee_keys, &public_key);
        let signer_refs =
            signer_refs_with_wallet_profile(&self.private_network_signer_refs, &profile);
        if let Err(error) =
            CommitteeRoster::from_public_keys_and_references(&committee_keys, &signer_refs)
        {
            self.session.notice = Some(format!("Wallet profile signer reference rejected: {error}"));
            return;
        }

        let signer_reference_already_exists =
            signer_refs_has_public_key(&self.private_network_signer_refs, &public_key);
        self.private_network_committee_keys = committee_keys;
        self.private_network_signer_refs = signer_refs;
        self.session.selected_view = View::Roles;

        match self.mark_neo_wallet_profile_used_and_reselect(&profile) {
            Ok(()) => {
                self.record_event(
                    None,
                    None,
                    EventKind::NeoWalletProfileUsed,
                    EventSeverity::Info,
                    format!(
                        "wallet-profile:{} applied to private network signer references",
                        profile.id
                    ),
                );
                self.session.notice = if signer_reference_already_exists {
                    Some(format!(
                        "Wallet profile signer reference already exists: {}",
                        profile.label
                    ))
                } else {
                    Some(format!(
                        "Wallet profile added to signer references: {}",
                        profile.label
                    ))
                };
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    fn mark_neo_wallet_profile_used_and_reselect(
        &mut self,
        profile: &NeoWalletProfile,
    ) -> anyhow::Result<()> {
        self.repository
            .mark_neo_wallet_profile_used(&profile.id, current_unix_time()?)?;
        self.selected_neo_wallet_profile = Some(profile.id.clone());
        self.reload_neo_wallet_profiles();
        self.selected_neo_wallet_profile = Some(profile.id.clone());
        Ok(())
    }
}
