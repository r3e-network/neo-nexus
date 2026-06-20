use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn reload_neo_wallet_profiles(&mut self) {
        match self.repository.list_neo_wallet_profiles() {
            Ok(profiles) => {
                self.neo_wallet_profiles = profiles;
                self.ensure_valid_neo_wallet_profile_selection();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn ensure_valid_neo_wallet_profile_selection(&mut self) {
        let selected_exists = self.selected_neo_wallet_profile.as_ref().is_some_and(|id| {
            self.neo_wallet_profiles
                .iter()
                .any(|profile| &profile.id == id)
        });
        if !selected_exists {
            self.selected_neo_wallet_profile = self
                .neo_wallet_profiles
                .first()
                .map(|profile| profile.id.clone());
        }
        self.wallet_profile_page = clamp_page(
            self.wallet_profile_page,
            self.filtered_neo_wallet_profiles().len(),
            WALLET_PROFILE_PAGE_SIZE,
        );
    }

    pub(in crate::app) fn neo_wallet_profile_filter(&self) -> NeoWalletProfileFilter {
        NeoWalletProfileFilter::new(
            self.wallet_profile_used_filter,
            self.wallet_profile_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_neo_wallet_profiles(&self) -> Vec<NeoWalletProfile> {
        filter_neo_wallet_profiles(&self.neo_wallet_profiles, &self.neo_wallet_profile_filter())
    }

    pub(in crate::app) fn selected_neo_wallet_profile(&self) -> Option<NeoWalletProfile> {
        let selected_id = self.selected_neo_wallet_profile.as_deref()?;
        self.neo_wallet_profiles
            .iter()
            .find(|profile| profile.id == selected_id)
            .cloned()
    }
}
