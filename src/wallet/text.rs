use super::{NeoWalletValidationReport, NeoWalletValidationStatus};

impl NeoWalletValidationReport {
    pub fn to_cli_text(&self) -> String {
        let mut text = format!(
            "wallet-validation: {}\nsource: {}\nversion: {}\naccounts: {}\nencrypted-accounts: {}\ndefault-accounts: {}\nwatch-only-accounts: {}\nchecks: {} passed, {} warnings, {} failed\n",
            self.status,
            self.source_path,
            self.wallet_version.as_deref().unwrap_or("-"),
            self.account_count,
            self.encrypted_account_count,
            self.default_account_count,
            self.watch_only_account_count,
            self.passed_count,
            self.warning_count,
            self.failed_count
        );
        if !self.contract_public_keys.is_empty() {
            text.push_str(&format!(
                "contract-public-keys: {}\n",
                self.contract_public_keys.join(", ")
            ));
        }
        for check in &self.checks {
            text.push_str(&format!(
                "{} [{}] {}: {}\n",
                check.status.cli_label(),
                check.category,
                check.label,
                check.message
            ));
        }
        text
    }
}

impl NeoWalletValidationStatus {
    fn cli_label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }
}
