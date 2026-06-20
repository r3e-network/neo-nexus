use serde_json::Value;

use crate::wallet::{NeoWalletValidationCheck, NeoWalletValidationStatus};

use super::{
    add_check,
    contract::{check_account_address_contract, check_contract},
};

mod address;
mod key;
mod model;

use address::check_account_address;
use key::{check_account_key, encrypted_account_key, watch_only_account_key};

pub(super) use model::AccountStats;

pub(super) fn check_account(
    checks: &mut Vec<NeoWalletValidationCheck>,
    account_index: usize,
    value: &Value,
) -> AccountStats {
    let label = format!("account-{account_index}");
    let Some(account) = value.as_object() else {
        add_check(
            checks,
            "account",
            &label,
            NeoWalletValidationStatus::Fail,
            "account entry must be a JSON object".to_string(),
        );
        return AccountStats::default();
    };

    let default = account
        .get("isDefault")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let key = account
        .get("key")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let address_payload = check_account_address(checks, &label, account.get("address"));
    check_account_key(checks, &label, key);
    let contract = check_contract(checks, &label, account.get("contract"));
    check_account_address_contract(
        checks,
        &label,
        address_payload.as_deref(),
        contract.script_hash.as_deref(),
    );

    AccountStats {
        encrypted: encrypted_account_key(key),
        default,
        watch_only: watch_only_account_key(key),
        contract_public_key: contract.public_key,
    }
}
