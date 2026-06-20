use std::collections::BTreeSet;

use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn refresh_private_network_launch_pack_sidecars(&mut self) {
        let Some(root_path) = self.private_network_last_export_root.clone() else {
            self.notice = Some("Export a private launch pack before loading sidecars".to_string());
            return;
        };

        match PrivateNetworkLaunchPackVerifier::sidecar_report(&root_path) {
            Ok(report) => {
                let sidecar_ids = report
                    .sidecars
                    .iter()
                    .map(|sidecar| sidecar.process.id.clone())
                    .collect::<BTreeSet<_>>();
                let removed_sidecar_ids = self
                    .private_network_sidecar_pids
                    .keys()
                    .filter(|id| !sidecar_ids.contains(*id))
                    .cloned()
                    .collect::<Vec<_>>();
                for process_id in &removed_sidecar_ids {
                    self.watchdog.clear(process_id);
                }
                self.private_network_sidecar_pids
                    .retain(|id, _pid| sidecar_ids.contains(id));

                let count = report.sidecar_count;
                self.private_network_sidecar_report = Some(report);
                self.notice = Some(format!(
                    "{} signer sidecar spec loaded from {}",
                    count,
                    short_path(&root_path, 54)
                ));
            }
            Err(error) => {
                self.private_network_sidecar_report = None;
                self.notice = Some(format!("Signer sidecar spec load failed: {error}"));
            }
        }
    }
}
