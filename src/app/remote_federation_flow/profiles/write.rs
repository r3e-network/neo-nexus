use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn create_remote_server_profile(&mut self) {
        match self
            .repository
            .create_remote_server(self.remote_server_draft())
        {
            Ok(profile) => {
                self.selected_remote_server = Some(profile.id.clone());
                let message = format!("Remote federation profile created: {}", profile.name);
                self.record_event(
                    Some(profile.id.clone()),
                    Some(profile.name.clone()),
                    EventKind::RemoteServerCreated,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.session.notice = Some(message);
                self.reload_remote_servers();
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn update_selected_remote_server_profile(&mut self) {
        let Some(profile) = self.selected_remote_server_profile() else {
            self.session.notice = Some("Select a remote server profile before updating".to_string());
            return;
        };
        match self
            .repository
            .update_remote_server(&profile.id, self.remote_server_draft())
        {
            Ok(updated) => {
                self.selected_remote_server = Some(updated.id.clone());
                let message = format!("Remote federation profile updated: {}", updated.name);
                self.record_event(
                    Some(updated.id.clone()),
                    Some(updated.name.clone()),
                    EventKind::RemoteServerUpdated,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.session.notice = Some(message);
                self.reload_remote_servers();
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn toggle_selected_remote_server_enabled(&mut self) {
        let Some(profile) = self.selected_remote_server_profile() else {
            self.session.notice = Some("Select a remote server profile before toggling it".to_string());
            return;
        };
        match self
            .repository
            .set_remote_server_enabled(&profile.id, !profile.enabled)
        {
            Ok(updated) => {
                self.selected_remote_server = Some(updated.id.clone());
                let message = format!(
                    "{} {}",
                    updated.name,
                    if updated.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
                self.record_event(
                    Some(updated.id.clone()),
                    Some(updated.name.clone()),
                    EventKind::RemoteServerUpdated,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.session.notice = Some(message);
                self.reload_remote_servers();
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
