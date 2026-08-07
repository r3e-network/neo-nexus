//! A minimal JSON-RPC client for chain-state reads.
//!
//! Separate from `rpc_health`'s prober: that one asks whether a node answers,
//! this one asks the node questions with parameters and cares about the shape
//! of the answer.

use std::time::Duration;

use serde_json::{json, Value};

use super::model::ChainQueryError;

pub(super) fn agent(timeout: Duration) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build()
}

/// Calls a JSON-RPC method and returns its `result`.
pub(super) fn call(
    agent: &ureq::Agent,
    endpoint: &str,
    method: &str,
    params: Value,
) -> Result<Value, ChainQueryError> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": "neonexus-chain-state",
        "method": method,
        "params": params,
    });
    let response = agent
        .post(endpoint)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|error| ChainQueryError::Unreachable(format!("{method}: {error}")))?;
    let text = response
        .into_string()
        .map_err(|error| ChainQueryError::Unreachable(format!("{method}: {error}")))?;
    parse_result(method, &text)
}

/// Splits a JSON-RPC envelope into `result` or a typed failure.
pub(super) fn parse_result(method: &str, text: &str) -> Result<Value, ChainQueryError> {
    let json: Value = serde_json::from_str(text).map_err(|error| {
        ChainQueryError::Unexpected(format!("{method} returned non-JSON: {error}"))
    })?;
    if let Some(error) = json.get("error") {
        // A node that answers with an error is reachable; the request was
        // wrong or unsupported, which is a different problem for the operator.
        return Err(ChainQueryError::Unexpected(format!(
            "{method} returned error: {error}"
        )));
    }
    json.get("result")
        .cloned()
        .ok_or_else(|| ChainQueryError::Unexpected(format!("{method} response has no result")))
}

/// Reads the `stack` of an `invokefunction` result, rejecting a FAULTed VM run.
pub(super) fn invocation_stack(method: &str, result: &Value) -> Result<Value, ChainQueryError> {
    let state = result.get("state").and_then(Value::as_str).unwrap_or("");
    if state != "HALT" {
        let exception = result
            .get("exception")
            .and_then(Value::as_str)
            .unwrap_or("no exception reported");
        return Err(ChainQueryError::Unexpected(format!(
            "{method} did not complete: {state} ({exception})"
        )));
    }
    result
        .get("stack")
        .and_then(|stack| stack.get(0))
        .cloned()
        .ok_or_else(|| ChainQueryError::Unexpected(format!("{method} returned an empty stack")))
}

#[cfg(test)]
#[path = "../../tests/unit/chain_state/rpc/tests.rs"]
mod tests;
