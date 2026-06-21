use anyhow::Result;
use serde::Serialize;

use crate::core::security::{NeoWalletProfile, NeoWalletValidationReport};

use super::json_text;

#[derive(Debug, Serialize)]
struct WalletValidationJsonReport<'a> {
    schema_version: u32,
    status: &'a str,
    success: bool,
    report: &'a NeoWalletValidationReport,
}

pub(in crate::cli) fn wallet_validation_json_text(
    report: &NeoWalletValidationReport,
) -> Result<String> {
    json_text(&WalletValidationJsonReport {
        schema_version: 1,
        status: &report.status,
        success: report.is_success(),
        report,
    })
}

pub(in crate::cli) fn wallet_profile_import_text(profile: &NeoWalletProfile) -> String {
    [
        "wallet-profile-import: ok".to_string(),
        format!("profile-id: {}", profile.id),
        format!("label: {}", profile.label),
        format!("wallet: {}", profile.source_path),
        format!(
            "wallet-version: {}",
            profile.wallet_version.as_deref().unwrap_or("-")
        ),
        format!("primary-address: {}", profile.primary_address),
        format!(
            "contract-public-keys: {}",
            if profile.contract_public_keys.is_empty() {
                "-".to_string()
            } else {
                profile.contract_public_keys.join(", ")
            }
        ),
        format!("wallet-sha256: {}", profile.wallet_sha256),
        format!("accounts: {}", profile.account_count),
        format!("encrypted-accounts: {}", profile.encrypted_account_count),
        format!("default-accounts: {}", profile.default_account_count),
        format!("watch-only-accounts: {}", profile.watch_only_account_count),
        format!("validated-at-unix: {}", profile.validated_at_unix),
        "privacy: metadata-only-no-private-keys-passwords-or-wallet-bytes".to_string(),
        String::new(),
    ]
    .join("\n")
}
