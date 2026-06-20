use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn reload_remote_servers(&mut self) {
        match self.repository.list_remote_servers() {
            Ok(profiles) => {
                self.remote_servers = profiles;
                self.ensure_valid_remote_server_selection();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn ensure_valid_remote_server_selection(&mut self) {
        let selected_exists = self
            .selected_remote_server
            .as_ref()
            .is_some_and(|id| self.remote_servers.iter().any(|profile| &profile.id == id));
        if !selected_exists {
            self.selected_remote_server = self
                .remote_servers
                .first()
                .map(|profile| profile.id.clone());
            self.remote_server_page = 0;
            self.remote_probe_history_page = 0;
        }
        self.remote_server_page = clamp_page(
            self.remote_server_page,
            self.filtered_remote_server_profiles().len(),
            REMOTE_SERVER_PAGE_SIZE,
        );
    }

    pub(in crate::app) fn remote_server_profile_filter(&self) -> RemoteServerProfileFilter {
        RemoteServerProfileFilter::new(
            self.remote_server_enabled_filter,
            self.remote_server_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_remote_server_profiles(&self) -> Vec<RemoteServerProfile> {
        filter_remote_server_profiles(&self.remote_servers, &self.remote_server_profile_filter())
    }

    pub(in crate::app) fn selected_remote_server_profile(&self) -> Option<RemoteServerProfile> {
        let selected_id = self.selected_remote_server.as_deref()?;
        self.remote_servers
            .iter()
            .find(|profile| profile.id == selected_id)
            .cloned()
    }

    pub(in crate::app) fn selected_remote_server_probe(&self) -> Option<RemoteServerProbeRecord> {
        let profile = self.selected_remote_server.as_deref()?;
        if let Some(record) = self
            .last_remote_server_probe
            .as_ref()
            .filter(|report| report.remote_server_id == profile)
            .cloned()
        {
            return Some(record);
        }
        self.repository
            .latest_remote_server_probe(profile)
            .ok()
            .flatten()
    }

    pub(in crate::app) fn selected_remote_server_probe_history(
        &self,
    ) -> Vec<RemoteServerProbeRecord> {
        let Some(profile) = self.selected_remote_server.as_deref() else {
            return Vec::new();
        };
        self.repository
            .list_remote_server_probes(profile, REMOTE_PROBE_RETAIN_PER_PROFILE)
            .unwrap_or_default()
    }

    pub(in crate::app) fn remote_probe_history_filter(&self) -> RemoteProbeHistoryFilter {
        RemoteProbeHistoryFilter::new(
            self.remote_probe_history_status_filter,
            self.remote_probe_history_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_selected_remote_server_probe_history(
        &self,
    ) -> Vec<RemoteServerProbeRecord> {
        filter_remote_probe_history(
            &self.selected_remote_server_probe_history(),
            &self.remote_probe_history_filter(),
        )
    }

    pub(in crate::app) fn clamp_remote_probe_history_page(&mut self) {
        let history = self.filtered_selected_remote_server_probe_history();
        self.remote_probe_history_page = clamp_page(
            self.remote_probe_history_page,
            history.len(),
            REMOTE_PROBE_HISTORY_PAGE_SIZE,
        );
    }
}
