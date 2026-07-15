use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn check_private_network_launch_pack_sidecar_health(&mut self) {
        let Some(sidecar_report) = self.private_network_sidecar_report_or_refresh() else {
            return;
        };
        if sidecar_report.sidecars.is_empty() {
            self.private_network_sidecar_health_report = Some(SidecarEndpointHealthReport::empty());
            self.session.notice = Some("No signer sidecars are defined in the launch pack".to_string());
            return;
        }

        let checked_at_unix = match current_unix_time() {
            Ok(value) => value,
            Err(error) => {
                self.session.notice = Some(error.to_string());
                return;
            }
        };
        let results = sidecar_report
            .sidecars
            .iter()
            .map(|sidecar| probe_sidecar_endpoint_health(sidecar, SIGNER_ENDPOINT_HEALTH_TIMEOUT))
            .collect::<Vec<_>>();
        let health_report = SidecarEndpointHealthReport::from_results(
            checked_at_unix,
            sidecar_report.sidecar_count,
            results,
        );
        let notice = sidecar_health_notice(&health_report);
        let severity = sidecar_health_event_severity(&health_report);

        self.private_network_sidecar_health_report = Some(health_report);
        self.record_event(
            None,
            None,
            EventKind::PrivateNetworkSignerSidecarHealthChecked,
            severity,
            format!("sidecar health: {notice}"),
        );
        self.session.notice = Some(notice);
    }
}

fn sidecar_health_event_severity(report: &SidecarEndpointHealthReport) -> EventSeverity {
    if report.unreachable_count > 0 {
        EventSeverity::Critical
    } else if report.missing_endpoint_count > 0 {
        EventSeverity::Warning
    } else {
        EventSeverity::Info
    }
}
