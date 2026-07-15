use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn drain_remote_federation_results(&mut self) {
        while let Ok(result) = self.remote_federation_results.try_recv() {
            self.remote_federation_pending.remove(&result.profile.id);
            let Some(profile) = self
                .remote_servers
                .iter()
                .find(|profile| profile.id == result.profile.id)
                .cloned()
            else {
                continue;
            };
            if !profile.enabled {
                continue;
            }

            let previous_status = self
                .repository
                .latest_remote_server_probe(&profile.id)
                .ok()
                .flatten()
                .map(|record| record.status);
            let report = match result.report {
                Ok(report) => report,
                Err(error) => remote_probe_failure_report(&profile, &error),
            };
            let record = match self.persist_remote_server_probe(&report) {
                Ok(record) => record,
                Err(error) => {
                    self.session.notice = Some(error.to_string());
                    continue;
                }
            };
            let status = record.status;
            let message = record.message.clone();
            self.last_remote_server_probe = Some(record);
            if should_record_remote_probe_event(previous_status, status) {
                self.record_event(
                    Some(profile.id.clone()),
                    Some(profile.name.clone()),
                    EventKind::RemoteServerProbed,
                    remote_probe_event_severity(status),
                    format!("Automatic remote Federation probe: {message}"),
                );
            }
        }
    }
}
