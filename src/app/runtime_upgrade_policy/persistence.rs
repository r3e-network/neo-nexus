use crate::runtime::RuntimeUpgradePolicy;

use super::super::{NeoNexusApp, RuntimeUpgradePolicyDraft};

impl NeoNexusApp {
    pub(super) fn persist_runtime_upgrade_policy_state(
        &mut self,
        policy: RuntimeUpgradePolicy,
    ) -> anyhow::Result<()> {
        self.repository.save_runtime_upgrade_policy(&policy)?;
        self.runtime_upgrade_policy = policy;
        self.runtime_upgrade_policy_draft =
            RuntimeUpgradePolicyDraft::from_policy(&self.runtime_upgrade_policy);
        Ok(())
    }
}
