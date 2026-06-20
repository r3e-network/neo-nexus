use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn delete_selected_neo_wallet_profile(&mut self) {
        let Some(profile) = self.selected_neo_wallet_profile() else {
            self.notice = Some("Select a wallet profile first".to_string());
            return;
        };
        match self.repository.delete_neo_wallet_profile(&profile.id) {
            Ok(()) => {
                self.selected_neo_wallet_profile = None;
                self.reload_neo_wallet_profiles();
                self.record_event(
                    None,
                    None,
                    EventKind::NeoWalletProfileDeleted,
                    EventSeverity::Warning,
                    format!("wallet-profile:{} metadata deleted", profile.id),
                );
                self.notice = Some(format!("Wallet profile deleted: {}", profile.label));
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
