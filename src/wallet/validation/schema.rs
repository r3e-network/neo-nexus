use serde_json::Value;

use crate::wallet::{NeoWalletValidationCheck, NeoWalletValidationStatus};

use super::add_check;

pub(super) fn check_version(checks: &mut Vec<NeoWalletValidationCheck>, version: Option<&str>) {
    match version {
        Some(version @ ("1.0" | "3.0")) => add_check(
            checks,
            "schema",
            "version",
            NeoWalletValidationStatus::Pass,
            format!("NEP-6 wallet version {version}"),
        ),
        Some(other) if !other.trim().is_empty() => add_check(
            checks,
            "schema",
            "version",
            NeoWalletValidationStatus::Warn,
            format!("wallet version {other} is not a common NeoNexus NEP-6 version"),
        ),
        _ => add_check(
            checks,
            "schema",
            "version",
            NeoWalletValidationStatus::Fail,
            "NEP-6 wallet version is missing".to_string(),
        ),
    }
}

pub(super) fn check_scrypt(checks: &mut Vec<NeoWalletValidationCheck>, value: Option<&Value>) {
    let Some(scrypt) = value.and_then(Value::as_object) else {
        add_check(
            checks,
            "scrypt",
            "parameters",
            NeoWalletValidationStatus::Fail,
            "NEP-6 wallet scrypt parameters are missing".to_string(),
        );
        return;
    };

    let n = scrypt.get("n").and_then(Value::as_u64);
    let r = scrypt.get("r").and_then(Value::as_u64);
    let p = scrypt.get("p").and_then(Value::as_u64);
    let status = if n.is_some_and(|value| value > 1 && value.is_power_of_two())
        && r.is_some_and(|value| value > 0)
        && p.is_some_and(|value| value > 0)
    {
        NeoWalletValidationStatus::Pass
    } else {
        NeoWalletValidationStatus::Fail
    };
    add_check(
        checks,
        "scrypt",
        "parameters",
        status,
        format!(
            "n={}, r={}, p={}",
            n.map_or_else(|| "-".to_string(), |value| value.to_string()),
            r.map_or_else(|| "-".to_string(), |value| value.to_string()),
            p.map_or_else(|| "-".to_string(), |value| value.to_string())
        ),
    );
}
