use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn probe_selected_remote_server(&mut self) {
        let Some(profile) = self.selected_remote_server_profile() else {
            self.notice = Some("Select a remote server profile before probing it".to_string());
            return;
        };
        match RemoteFederationClient::probe(&profile, REMOTE_FEDERATION_TIMEOUT) {
            Ok(report) => self.record_successful_manual_probe(&profile, &report),
            Err(error) => self.record_failed_manual_probe(&profile, &error.to_string()),
        }
    }

    fn record_successful_manual_probe(
        &mut self,
        profile: &RemoteServerProfile,
        report: &RemoteServerProbeReport,
    ) {
        let record = match self.persist_remote_server_probe(report) {
            Ok(record) => record,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };
        let message = record.message.clone();
        let severity = remote_probe_event_severity(record.status);
        self.last_remote_server_probe = Some(record);
        self.record_event(
            Some(profile.id.clone()),
            Some(profile.name.clone()),
            EventKind::RemoteServerProbed,
            severity,
            message.clone(),
        );
        self.notice = Some(message);
    }

    fn record_failed_manual_probe(&mut self, profile: &RemoteServerProfile, error: &str) {
        let fallback_report = remote_probe_failure_report(profile, error);
        let message = fallback_report.message.clone();
        if let Ok(record) = self.persist_remote_server_probe(&fallback_report) {
            self.last_remote_server_probe = Some(record);
        }
        self.record_event(
            Some(profile.id.clone()),
            Some(profile.name.clone()),
            EventKind::RemoteServerProbed,
            EventSeverity::Critical,
            message.clone(),
        );
        self.notice = Some(message);
    }
}
