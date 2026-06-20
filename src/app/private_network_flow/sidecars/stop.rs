use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn stop_private_network_launch_pack_sidecars(&mut self) {
        if self.private_network_sidecar_pids.is_empty() {
            self.notice = Some("No signer sidecars are currently running".to_string());
            return;
        }

        let process_ids = self
            .private_network_sidecar_pids
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let mut stopped = 0usize;
        let mut stale = 0usize;
        for process_id in process_ids {
            match self.supervisor.stop_process(&process_id) {
                Ok(Some(stop)) => {
                    stopped += 1;
                    self.private_network_sidecar_pids.remove(&process_id);
                    self.watchdog.clear(&process_id);
                    self.record_event(
                        None,
                        None,
                        EventKind::PrivateNetworkSignerSidecarStopped,
                        EventSeverity::Info,
                        format!(
                            "signer-sidecar:{} stopped ({})",
                            process_id,
                            stop.operator_summary()
                        ),
                    );
                }
                Ok(None) => {
                    stale += 1;
                    self.private_network_sidecar_pids.remove(&process_id);
                    self.watchdog.clear(&process_id);
                }
                Err(error) => {
                    self.notice = Some(format!("Signer sidecar stop failed: {error}"));
                    return;
                }
            }
        }

        self.notice = Some(format!(
            "{} signer sidecar stopped; {} stale records cleared",
            stopped, stale
        ));
    }
}
