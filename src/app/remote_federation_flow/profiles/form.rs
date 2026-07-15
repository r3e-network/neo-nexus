use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn remote_server_draft(&self) -> NewRemoteServerProfile {
        NewRemoteServerProfile {
            name: self.remote_server_name.clone(),
            base_url: self.remote_server_base_url.clone(),
            description: self.remote_server_description.clone(),
            enabled: self.remote_server_enabled,
        }
    }

    pub(in crate::app) fn load_selected_remote_server_into_form(&mut self) {
        let Some(profile) = self.selected_remote_server_profile() else {
            self.session.notice = Some("Select a remote server profile first".to_string());
            return;
        };
        self.remote_server_name = profile.name.clone();
        self.remote_server_base_url = profile.base_url.clone();
        self.remote_server_description = profile.description.clone();
        self.remote_server_enabled = profile.enabled;
        self.session.notice = Some(format!("Remote profile loaded: {}", profile.name));
    }

    pub(in crate::app) fn reset_remote_server_form(&mut self) {
        let draft = NewRemoteServerProfile::default();
        self.remote_server_name = draft.name;
        self.remote_server_base_url = draft.base_url;
        self.remote_server_description = draft.description;
        self.remote_server_enabled = draft.enabled;
    }
}
