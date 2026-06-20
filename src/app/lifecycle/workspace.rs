mod metrics;
mod nodes;
mod policies;
mod reconciliation;

use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn reload_workspace_data(&mut self) {
        self.reload_nodes();
        self.reload_runtime_catalog_profiles();
        self.reload_runtime_signer_profiles();
        self.reload_neo_wallet_profiles();
        self.reload_remote_servers();
        self.refresh_metrics_now();
    }
}
