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

/// Confirms an endpoint speaks Neo N3 JSON-RPC before the N3-only reads run.
///
/// Governance and designation are N3-native methods (`getcommittee`,
/// `invokefunction` on a native hash). Pointed at a Neo X endpoint they would
/// each answer `-32601`, and the operator would read a pile of "unexpected"
/// failures instead of the one true sentence — that this endpoint is not a
/// Neo N3 node.
pub(super) fn require_neo_n3(agent: &ureq::Agent, endpoint: &str) -> Result<(), ChainQueryError> {
    n3_guard_verdict(call(agent, endpoint, "getversion", json!([])))
}

/// The guard's decision, separated from the network call so tests stay offline.
fn n3_guard_verdict(probe: Result<Value, ChainQueryError>) -> Result<(), ChainQueryError> {
    match probe {
        Ok(_) => Ok(()),
        Err(ChainQueryError::Unreachable(message)) => Err(ChainQueryError::Unreachable(message)),
        Err(ChainQueryError::Unexpected(_)) => Err(ChainQueryError::Unexpected(
            "the endpoint did not answer getversion: governance and designation reads exist \
             only on Neo N3, and a Neo X endpoint speaks Ethereum JSON-RPC instead"
                .to_string(),
        )),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/chain_state/rpc/tests.rs"]
mod tests;
