use super::*;

mod health;
mod refresh;
mod start;
mod stop;

impl NeoNexusApp {
    pub(in crate::app) fn private_network_sidecar_report_or_refresh(
        &mut self,
    ) -> Option<PrivateNetworkLaunchPackSidecarReport> {
        if self.private_network_sidecar_report.is_none() {
            self.refresh_private_network_launch_pack_sidecars();
        }
        self.private_network_sidecar_report.clone()
    }
}
