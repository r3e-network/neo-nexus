use serde_json::Value;

use crate::{
    redaction::redact_sensitive_text,
    wallet::{
        crypto::looks_like_plain_private_key, NeoWalletValidationCheck, NeoWalletValidationStatus,
    },
};

use super::add_check;

pub(super) fn check_secret_boundary(checks: &mut Vec<NeoWalletValidationCheck>, value: &Value) {
    let mut findings = Vec::new();
    collect_secret_findings(value, "$", &mut findings);
    add_check(
        checks,
        "secret-boundary",
        "plaintext-material",
        if findings.is_empty() {
            NeoWalletValidationStatus::Pass
        } else {
            NeoWalletValidationStatus::Fail
        },
        if findings.is_empty() {
            "no plaintext private key, password, mnemonic, seed, token, or webhook markers"
                .to_string()
        } else {
            format!("plaintext secret markers found: {}", findings.join(", "))
        },
    );
}

fn collect_secret_findings(value: &Value, path: &str, findings: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = if path == "$" {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                if child_path.ends_with(".key") && key == "key" {
                    check_account_key_secret_value(child, &child_path, findings);
                } else {
                    if sensitive_wallet_key(key) {
                        findings.push(child_path.clone());
                    }
                    collect_secret_findings(child, &child_path, findings);
                }
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                collect_secret_findings(child, &format!("{path}[{index}]"), findings);
            }
        }
        Value::String(text) => {
            if !text.trim().is_empty() && redact_sensitive_text(text) != *text {
                findings.push(path.to_string());
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn check_account_key_secret_value(value: &Value, path: &str, findings: &mut Vec<String>) {
    if value
        .as_str()
        .is_some_and(|key| looks_like_plain_private_key(key.trim()))
    {
        findings.push(path.to_string());
    }
}

fn sensitive_wallet_key(key: &str) -> bool {
    let normalized = key
        .trim()
        .trim_start_matches('-')
        .to_ascii_lowercase()
        .replace('_', "-");
    normalized.contains("password")
        || normalized.contains("passphrase")
        || normalized.contains("private-key")
        || normalized.contains("privatekey")
        || normalized == "wif"
        || normalized.contains("mnemonic")
        || normalized.ends_with("-seed")
        || normalized == "seed"
        || normalized.contains("api-key")
        || normalized.contains("apikey")
        || normalized.ends_with("-token")
        || normalized == "token"
        || normalized.contains("secret")
        || normalized.contains("webhook")
}
