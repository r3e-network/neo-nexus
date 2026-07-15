use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn delete_selected_runtime_signer_profile(&mut self) {
        let Some(profile) = self.selected_runtime_signer_profile() else {
            self.session.notice = Some("Select a trusted signer first".to_string());
            return;
        };
        match self.repository.delete_runtime_signer_profile(&profile.id) {
            Ok(()) => {
                self.selected_runtime_signer_profile = None;
                self.reload_runtime_signer_profiles();
                self.session.notice = Some(format!("Trusted signer deleted: {}", profile.label));
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
