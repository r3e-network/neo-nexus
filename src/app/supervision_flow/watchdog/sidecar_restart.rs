use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn restart_private_network_sidecar_by_id(
        &mut self,
        process_id: &str,
        attempt: u32,
    ) {
        let Some(report) = self.private_network_sidecar_report_or_refresh() else {
            self.watchdog.clear(process_id);
            return;
        };
        let Some(sidecar) = report
            .sidecars
            .iter()
            .find(|sidecar| sidecar.process.id == process_id)
            .cloned()
        else {
            self.watchdog.clear(process_id);
            let message = format!("signer-sidecar:{process_id} watchdog skipped; spec not loaded");
            self.record_event_notice(EventKind::WatchdogSkipped, EventSeverity::Warning, message);
            return;
        };

        if self.private_network_sidecar_pids.contains_key(process_id) {
            return;
        }

        if let Some(finding) = sidecar_execution_policy_finding(
            &report,
            &sidecar,
            self.private_network_allow_external_sidecars,
        ) {
            let message = format!(
                "signer-sidecar:{process_id} watchdog restart blocked by sidecar execution policy: {}",
                finding.summary()
            );
            self.record_event_notice(
                EventKind::PrivateNetworkSignerSidecarExecutionBlocked,
                EventSeverity::Warning,
                message,
            );
            return;
        }

        match self
            .supervisor
            .start_process(&sidecar.process, &sidecar.log_path)
        {
            Ok(start) => {
                self.private_network_sidecar_pids
                    .insert(process_id.to_string(), start.pid);
                let message = format!(
                    "signer-sidecar:{} restarted by watchdog attempt {} with PID {}; log {}",
                    sidecar.signer_label,
                    attempt,
                    start.pid,
                    short_path(&start.log_path, 42)
                );
                self.record_event_notice(
                    EventKind::WatchdogRestarted,
                    EventSeverity::Warning,
                    message,
                );
            }
            Err(error) => {
                let message = format!(
                    "signer-sidecar:{} watchdog restart failed: {error}",
                    sidecar.signer_label
                );
                self.record_event_notice(
                    EventKind::PrivateNetworkSignerSidecarStartFailed,
                    EventSeverity::Critical,
                    message,
                );
            }
        }
    }
}
