use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn validate_private_network_launch_pack(
        &mut self,
        root_path: &std::path::Path,
    ) -> String {
        match PrivateNetworkLaunchPackVerifier::validate(root_path) {
            Ok(validation) => self.record_private_launch_pack_validation(validation),
            Err(error) => self.record_private_launch_pack_validation_failure(error),
        }
    }

    fn record_private_launch_pack_validation(
        &mut self,
        validation: PrivateNetworkLaunchPackValidation,
    ) -> String {
        let mut validation_message = private_launch_pack_validation_notice(&validation);
        validation_message.push_str(&validation_report_suffix(&validation));
        self.record_event(
            None,
            None,
            EventKind::PrivateNetworkLaunchPackValidated,
            launch_pack_validation_severity(&validation),
            validation_message.clone(),
        );
        self.private_network_last_validation = Some(validation);
        validation_message
    }

    fn record_private_launch_pack_validation_failure(&mut self, error: anyhow::Error) -> String {
        let validation_message = format!("Private launch pack validation failed to run: {error}");
        self.record_event(
            None,
            None,
            EventKind::PrivateNetworkLaunchPackValidated,
            EventSeverity::Critical,
            validation_message.clone(),
        );
        self.private_network_last_validation = None;
        validation_message
    }
}

fn validation_report_suffix(validation: &PrivateNetworkLaunchPackValidation) -> String {
    match validation.write_reports() {
        Ok(report) => format!("; report {}", short_path(&report.text_path, 48)),
        Err(error) => format!("; validation report write failed: {error}"),
    }
}
