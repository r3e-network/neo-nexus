use std::path::Path;

use anyhow::Result;

use super::*;

impl CommitteeRoster {
    pub fn sidecar_processes(
        &self,
        launch_pack_root: impl AsRef<Path>,
    ) -> Result<Vec<CommitteeSidecarProcess>> {
        let launch_pack_root = launch_pack_root.as_ref();
        self.signers
            .iter()
            .filter_map(|signer| {
                signer
                    .signer_command_plan
                    .as_ref()
                    .map(|plan| (signer, plan))
            })
            .map(|(signer, plan)| signer.sidecar_process(launch_pack_root, plan))
            .collect()
    }
}

impl CommitteeSigner {
    fn sidecar_process(
        &self,
        launch_pack_root: &Path,
        plan: &SignerCommandPlan,
    ) -> Result<CommitteeSidecarProcess> {
        committee_sidecar_process(
            launch_pack_root,
            &self.label,
            &self.public_key,
            self.wallet_path.clone(),
            self.signer_endpoint.clone(),
            plan,
        )
    }
}
