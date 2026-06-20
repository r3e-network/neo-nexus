use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::Value;

use super::{NeoWalletValidationCheck, NeoWalletValidationReport, NeoWalletValidationStatus};

mod account;
mod contract;
mod report;
mod schema;
mod secret_boundary;

use account::check_account;
use report::{wallet_report, WalletStats};
use schema::{check_scrypt, check_version};
use secret_boundary::check_secret_boundary;

pub(super) fn validate_path(path: impl AsRef<Path>) -> Result<NeoWalletValidationReport> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read Neo wallet {}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse Neo wallet {}", path.display()))?;
    Ok(validate_wallet_value(&value, &path.display().to_string()))
}

pub(super) fn validate_wallet_value(value: &Value, source_path: &str) -> NeoWalletValidationReport {
    let mut checks = Vec::new();
    let mut stats = WalletStats::default();

    let Some(root) = value.as_object() else {
        add_check(
            &mut checks,
            "schema",
            "root",
            NeoWalletValidationStatus::Fail,
            "NEP-6 wallet must be a JSON object".to_string(),
        );
        return wallet_report(source_path, None, stats, checks);
    };

    let wallet_version = root
        .get("version")
        .and_then(Value::as_str)
        .map(str::to_string);
    check_version(&mut checks, wallet_version.as_deref());
    check_scrypt(&mut checks, root.get("scrypt"));
    check_secret_boundary(&mut checks, value);

    match root.get("accounts").and_then(Value::as_array) {
        Some(accounts) if !accounts.is_empty() => {
            stats.account_count = accounts.len();
            add_check(
                &mut checks,
                "accounts",
                "count",
                NeoWalletValidationStatus::Pass,
                format!("{} account(s)", accounts.len()),
            );
            for (index, account) in accounts.iter().enumerate() {
                stats.add_account(check_account(&mut checks, index + 1, account));
            }
        }
        Some(_) => add_check(
            &mut checks,
            "accounts",
            "count",
            NeoWalletValidationStatus::Fail,
            "NEP-6 wallet must contain at least one account".to_string(),
        ),
        None => add_check(
            &mut checks,
            "accounts",
            "count",
            NeoWalletValidationStatus::Fail,
            "NEP-6 wallet accounts array is missing".to_string(),
        ),
    }

    add_check(
        &mut checks,
        "accounts",
        "encrypted-accounts",
        if stats.encrypted_account_count > 0 {
            NeoWalletValidationStatus::Pass
        } else {
            NeoWalletValidationStatus::Fail
        },
        format!("{} encrypted account(s)", stats.encrypted_account_count),
    );
    add_check(
        &mut checks,
        "accounts",
        "default-account",
        if stats.default_account_count == 1 {
            NeoWalletValidationStatus::Pass
        } else {
            NeoWalletValidationStatus::Warn
        },
        format!("{} default account(s)", stats.default_account_count),
    );

    wallet_report(source_path, wallet_version, stats, checks)
}

pub(super) fn add_check(
    checks: &mut Vec<NeoWalletValidationCheck>,
    category: &str,
    label: impl Into<String>,
    status: NeoWalletValidationStatus,
    message: String,
) {
    checks.push(NeoWalletValidationCheck {
        category: category.to_string(),
        label: label.into(),
        status,
        message,
    });
}
