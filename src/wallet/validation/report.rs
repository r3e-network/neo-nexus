use crate::wallet::{
    NeoWalletValidationCheck, NeoWalletValidationReport, NeoWalletValidationStatus,
};

use super::account::AccountStats;

#[derive(Default)]
pub(super) struct WalletStats {
    pub(super) account_count: usize,
    pub(super) encrypted_account_count: usize,
    pub(super) default_account_count: usize,
    pub(super) watch_only_account_count: usize,
    pub(super) contract_public_keys: Vec<String>,
}

impl WalletStats {
    pub(super) fn add_account(&mut self, account: AccountStats) {
        if account.encrypted {
            self.encrypted_account_count += 1;
        }
        if account.default {
            self.default_account_count += 1;
        }
        if account.watch_only {
            self.watch_only_account_count += 1;
        }
        if let Some(public_key) = account.contract_public_key {
            self.contract_public_keys.push(public_key);
        }
    }
}

pub(super) fn wallet_report(
    source_path: &str,
    wallet_version: Option<String>,
    stats: WalletStats,
    checks: Vec<NeoWalletValidationCheck>,
) -> NeoWalletValidationReport {
    let passed_count = checks
        .iter()
        .filter(|check| check.status == NeoWalletValidationStatus::Pass)
        .count();
    let warning_count = checks
        .iter()
        .filter(|check| check.status == NeoWalletValidationStatus::Warn)
        .count();
    let failed_count = checks
        .iter()
        .filter(|check| check.status == NeoWalletValidationStatus::Fail)
        .count();
    NeoWalletValidationReport {
        schema_version: 1,
        status: if failed_count == 0 { "ok" } else { "failed" }.to_string(),
        source_path: source_path.to_string(),
        wallet_version,
        account_count: stats.account_count,
        encrypted_account_count: stats.encrypted_account_count,
        default_account_count: stats.default_account_count,
        watch_only_account_count: stats.watch_only_account_count,
        contract_public_keys: stats.contract_public_keys,
        passed_count,
        warning_count,
        failed_count,
        checks,
    }
}
