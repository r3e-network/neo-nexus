use super::*;

impl CommitteeHandoffSummary {
    pub fn status_label(&self) -> &'static str {
        if self.signer_count == 0 {
            "missing"
        } else if self.missing_required_signer_count > 0 {
            "incomplete"
        } else if self.missing_wallet_reference_count > 0 {
            "wallets pending"
        } else if self.sidecar_command_count > 0 {
            "sidecars planned"
        } else if self.endpoint_reference_count > 0 {
            "endpoints planned"
        } else {
            "wallets ready"
        }
    }

    pub fn operator_summary(&self) -> String {
        format!(
            "{}: {}/{} signers, {} wallets ({} missing), {} endpoints, {} sidecars",
            self.status_label(),
            self.signer_count,
            self.required_signer_count,
            self.wallet_reference_count,
            self.missing_wallet_reference_count,
            self.endpoint_reference_count,
            self.sidecar_command_count
        )
    }
}

impl CommitteeRoster {
    pub fn handoff_summary(&self, required_signer_count: usize) -> CommitteeHandoffSummary {
        let signer_count = self.signers.len();
        let wallet_reference_count = self
            .signers
            .iter()
            .filter(|signer| signer.wallet_path.is_some())
            .count();
        let endpoint_reference_count = self
            .signers
            .iter()
            .filter(|signer| signer.signer_endpoint.is_some())
            .count();
        let sidecar_command_count = self
            .signers
            .iter()
            .filter(|signer| signer.signer_command.is_some())
            .count();
        let sidecar_command_plan_count = self
            .signers
            .iter()
            .filter(|signer| signer.signer_command_plan.is_some())
            .count();

        CommitteeHandoffSummary {
            required_signer_count,
            signer_count,
            missing_required_signer_count: required_signer_count.saturating_sub(signer_count),
            wallet_reference_count,
            missing_wallet_reference_count: signer_count.saturating_sub(wallet_reference_count),
            endpoint_reference_count,
            sidecar_command_count,
            sidecar_command_plan_count,
        }
    }
}
