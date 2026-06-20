use serde_json::Value;

pub(super) fn primary_account_address(value: &Value) -> Option<String> {
    let accounts = value.get("accounts").and_then(Value::as_array)?;
    accounts
        .iter()
        .find(|account| {
            account
                .get("isDefault")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .and_then(account_address)
        .or_else(|| accounts.iter().find_map(account_address))
        .map(|address| address.trim().to_string())
}

fn account_address(account: &Value) -> Option<&str> {
    account
        .get("address")
        .and_then(Value::as_str)
        .filter(|address| !address.trim().is_empty())
}
