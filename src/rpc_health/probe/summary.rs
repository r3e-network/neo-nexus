use anyhow::Result;
use serde_json::Value;

use crate::rpc_health::RpcMethodHealth;

pub(super) fn method_health(method: &'static str, result: &Result<Value>) -> RpcMethodHealth {
    match result {
        Ok(value) => RpcMethodHealth {
            method,
            ok: true,
            detail: summarize_value(value),
        },
        Err(error) => RpcMethodHealth {
            method,
            ok: false,
            detail: error.to_string(),
        },
    }
}

pub(super) fn summarize_version(value: &Value) -> Option<String> {
    if let Some(user_agent) = value.get("useragent").and_then(Value::as_str) {
        return Some(user_agent.to_string());
    }
    if let Some(user_agent) = value.get("user_agent").and_then(Value::as_str) {
        return Some(user_agent.to_string());
    }
    if let Some(version) = value.get("version").and_then(Value::as_str) {
        return Some(version.to_string());
    }
    if value.is_object() {
        return Some(summarize_value(value));
    }
    value.as_str().map(ToString::to_string)
}

pub(super) fn parse_block_count(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse::<u64>().ok()))
}

fn summarize_value(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return single_line(text);
    }
    if let Some(number) = value.as_u64() {
        return number.to_string();
    }
    compact_json(value)
}

pub(super) fn compact_json(value: &Value) -> String {
    single_line(&value.to_string())
}

fn single_line(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(220)
        .collect()
}
