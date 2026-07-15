use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn start_private_network_launch_pack_sidecars(&mut self) {
        let Some(report) = self.private_network_sidecar_report_or_refresh() else {
            return;
        };
        if report.sidecars.is_empty() {
            self.session.notice = Some("No signer sidecars are defined in the launch pack".to_string());
            return;
        }

        let policy_findings = sidecar_execution_policy_findings(
            &report,
            self.private_network_allow_external_sidecars,
        );
        if let Some(first_finding) = policy_findings.first() {
            let message = format!(
                "{} signer sidecar start blocked by sidecar execution policy: {}",
                policy_findings.len(),
                first_finding.summary()
            );
            self.record_event_notice(
                EventKind::PrivateNetworkSignerSidecarExecutionBlocked,
                EventSeverity::Warning,
                message,
            );
            return;
        }

        let mut started = 0usize;
        let mut already_running = 0usize;
        let mut last_log_path = None;
        for sidecar in &report.sidecars {
            if self
                .private_network_sidecar_pids
                .contains_key(&sidecar.process.id)
            {
                already_running += 1;
                continue;
            }

            match self
                .supervisor
                .start_process(&sidecar.process, &sidecar.log_path)
            {
                Ok(start) => {
                    started += 1;
                    last_log_path = Some(start.log_path.clone());
                    self.private_network_sidecar_pids
                        .insert(sidecar.process.id.clone(), start.pid);
                    self.watchdog.clear(&sidecar.process.id);
                    self.record_event(
                        None,
                        None,
                        EventKind::PrivateNetworkSignerSidecarStarted,
                        EventSeverity::Info,
                        format!(
                            "signer-sidecar:{} started with PID {} from {}",
                            sidecar.signer_label,
                            start.pid,
                            short_path(&report.manifest_path, 54)
                        ),
                    );
                }
                Err(error) => {
                    let message = format!(
                        "signer-sidecar:{} start failed: {error}",
                        sidecar.signer_label
                    );
                    self.record_event_notice(
                        EventKind::PrivateNetworkSignerSidecarStartFailed,
                        EventSeverity::Critical,
                        message,
                    );
                    return;
                }
            }
        }

        self.session.notice = sidecar_start_notice(started, already_running, last_log_path.as_ref());
    }
}

fn sidecar_start_notice(
    started: usize,
    already_running: usize,
    last_log_path: Option<&std::path::PathBuf>,
) -> Option<String> {
    if started == 0 {
        Some(format!(
            "No signer sidecars started; {} already running",
            already_running
        ))
    } else if let Some(log_path) = last_log_path {
        Some(format!(
            "{} signer sidecar started; {} already running; log {}",
            started,
            already_running,
            short_path(log_path, 42)
        ))
    } else {
        Some(format!(
            "{} signer sidecar started; {} already running",
            started, already_running
        ))
    }
}
