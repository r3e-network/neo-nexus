use serde_json::Value;

use crate::wallet::{
    crypto::{extract_single_sig_contract_public_key, is_even_hex, script_hash_from_hex},
    NeoWalletValidationCheck, NeoWalletValidationStatus,
};

use super::add_check;

#[derive(Default)]
pub(super) struct ContractStats {
    pub(super) public_key: Option<String>,
    pub(super) script_hash: Option<Vec<u8>>,
}

pub(super) fn check_contract(
    checks: &mut Vec<NeoWalletValidationCheck>,
    label: &str,
    value: Option<&Value>,
) -> ContractStats {
    let Some(contract) = value.and_then(Value::as_object) else {
        add_check(
            checks,
            "account-contract",
            label,
            NeoWalletValidationStatus::Warn,
            "account contract metadata is missing".to_string(),
        );
        return ContractStats::default();
    };

    let script = contract.get("script").and_then(Value::as_str);
    match script {
        Some(script) if is_even_hex(script) => {
            let public_key = extract_single_sig_contract_public_key(script);
            let script_hash = script_hash_from_hex(script);
            add_check(
                checks,
                "account-contract",
                label,
                NeoWalletValidationStatus::Pass,
                match public_key.as_deref() {
                    Some(public_key) => format!("contract script binds public key {public_key}"),
                    None => format!("contract script has {} hex chars", script.len()),
                },
            );
            ContractStats {
                public_key,
                script_hash,
            }
        }
        Some(_) => {
            add_check(
                checks,
                "account-contract",
                label,
                NeoWalletValidationStatus::Fail,
                "contract script must be even-length hexadecimal".to_string(),
            );
            ContractStats::default()
        }
        None => {
            add_check(
                checks,
                "account-contract",
                label,
                NeoWalletValidationStatus::Warn,
                "account contract script is missing".to_string(),
            );
            ContractStats::default()
        }
    }
}

pub(super) fn check_account_address_contract(
    checks: &mut Vec<NeoWalletValidationCheck>,
    label: &str,
    address_payload: Option<&[u8]>,
    script_hash: Option<&[u8]>,
) {
    let (Some(address_payload), Some(script_hash)) = (address_payload, script_hash) else {
        return;
    };
    let Some(address_hash) = address_payload.get(1..) else {
        return;
    };
    let matches_script = address_hash == script_hash;
    add_check(
        checks,
        "account-address-contract",
        label,
        if matches_script {
            NeoWalletValidationStatus::Pass
        } else {
            NeoWalletValidationStatus::Fail
        },
        if matches_script {
            "account address matches contract script hash".to_string()
        } else {
            "account address does not match contract script hash".to_string()
        },
    );
}
