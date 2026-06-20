use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::wallet::{
    crypto::sha256_hex, profile::primary::primary_account_address,
    validation::validate_wallet_value, NeoWalletProfile,
};

use super::validate_neo_wallet_profile;

pub(super) fn profile_from_path(
    path: impl AsRef<Path>,
    id: impl Into<String>,
    label: impl Into<String>,
    validated_at_unix: u64,
) -> Result<NeoWalletProfile> {
    let path = path.as_ref();
    let bytes =
        fs::read(path).with_context(|| format!("failed to read Neo wallet {}", path.display()))?;
    let value: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse Neo wallet {}", path.display()))?;
    let source_path = path.display().to_string();
    let report = validate_wallet_value(&value, &source_path);
    if !report.is_success() {
        anyhow::bail!(
            "Neo wallet profile import requires a valid encrypted wallet; {} validation check(s) failed",
            report.failed_count
        );
    }
    let primary_address = primary_account_address(&value)
        .context("Neo wallet profile import could not determine primary account address")?;
    let profile = NeoWalletProfile {
        id: id.into().trim().to_string(),
        label: label.into().trim().to_string(),
        source_path,
        wallet_version: report.wallet_version,
        primary_address,
        contract_public_keys: report.contract_public_keys,
        wallet_sha256: sha256_hex(&bytes),
        account_count: report.account_count,
        encrypted_account_count: report.encrypted_account_count,
        default_account_count: report.default_account_count,
        watch_only_account_count: report.watch_only_account_count,
        validated_at_unix,
        last_used_at_unix: None,
    };
    validate_neo_wallet_profile(&profile)?;
    Ok(profile)
}
