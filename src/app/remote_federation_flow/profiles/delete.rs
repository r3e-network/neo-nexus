use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn delete_selected_remote_server(&mut self) {
        let Some(profile) = self.selected_remote_server_profile() else {
            self.notice = Some("Select a remote server profile before deleting it".to_string());
            return;
        };
        match self.repository.delete_remote_server(&profile.id) {
            Ok(()) => {
                let message = format!("Remote federation profile deleted: {}", profile.name);
                self.record_event(
                    Some(profile.id.clone()),
                    Some(profile.name.clone()),
                    EventKind::RemoteServerDeleted,
                    EventSeverity::Warning,
                    message.clone(),
                );
                self.notice = Some(message);
                self.selected_remote_server = None;
                if self
                    .last_remote_server_probe
                    .as_ref()
                    .is_some_and(|report| report.remote_server_id == profile.id)
                {
                    self.last_remote_server_probe = None;
                }
                self.remote_federation_pending.remove(&profile.id);
                self.remote_federation_last_started.remove(&profile.id);
                self.reload_remote_servers();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
