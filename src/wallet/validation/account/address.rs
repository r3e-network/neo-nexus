use serde_json::Value;

use crate::wallet::{
    crypto::neo_address_payload, NeoWalletValidationCheck, NeoWalletValidationStatus,
};

use super::super::add_check;

pub(super) fn check_account_address(
    checks: &mut Vec<NeoWalletValidationCheck>,
    label: &str,
    value: Option<&Value>,
) -> Option<Vec<u8>> {
    let address = value.and_then(Value::as_str).unwrap_or_default();
    let payload = neo_address_payload(address);
    let status = if payload.is_some() {
        NeoWalletValidationStatus::Pass
    } else {
        NeoWalletValidationStatus::Fail
    };
    add_check(
        checks,
        "account-address",
        label,
        status,
        if address.is_empty() {
            "account address is missing".to_string()
        } else {
            address.to_string()
        },
    );
    payload
}
