//! Whether the committee has designated a node's key for a role.
//!
//! `RoleManagement.getDesignatedByRole(role, index)` is a read-only contract
//! call, so this needs nothing but an RPC endpoint. Designating a key is not
//! read-only — it is a committee-witnessed transaction — and NeoNexus does not
//! do it. Reporting the truth is the whole job here.

use std::time::Duration;

use serde_json::{json, Value};

use crate::roles::ChainRole;

use super::{
    model::{ChainQueryError, DesignationStatus, RoleDesignation},
    rpc::{agent, call, invocation_stack},
};

/// The `RoleManagement` native contract. Its hash is fixed by the protocol and
/// is the same on every Neo N3 network.
pub const ROLE_MANAGEMENT_HASH: &str = "0x49cf4e5378ffcd4dec034fd98a174c5491e395e2";

/// Asks a node which keys hold `role`, and whether `node_public_key` is one.
///
/// Pass `None` for the key when the node's signing key is unknown: the answer
/// then reports who holds the role without claiming anything about this node.
pub fn designation_status(
    endpoint: &str,
    role: ChainRole,
    node_public_key: Option<&str>,
    timeout: Duration,
) -> DesignationStatus {
    let agent = agent(timeout);
    let height = block_count(&agent, endpoint)?;
    let result = call(
        &agent,
        endpoint,
        "invokefunction",
        json!([
            ROLE_MANAGEMENT_HASH,
            "getDesignatedByRole",
            [
                { "type": "Integer", "value": role.on_chain_value().to_string() },
                { "type": "Integer", "value": height.to_string() },
            ]
        ]),
    )?;
    let designated = parse_designated(&invocation_stack("getDesignatedByRole", &result)?)?;
    Ok(RoleDesignation {
        role,
        includes_node_key: node_public_key
            .map(|key| designated.iter().any(|designated| designated == key)),
        designated,
    })
}

fn block_count(agent: &ureq::Agent, endpoint: &str) -> Result<u64, ChainQueryError> {
    call(agent, endpoint, "getblockcount", json!([]))?
        .as_u64()
        .ok_or_else(|| ChainQueryError::Unexpected("getblockcount is not a number".to_string()))
}

/// The contract returns an array of ByteString items, each a base64-encoded
/// compressed public key.
pub(super) fn parse_designated(item: &Value) -> Result<Vec<String>, ChainQueryError> {
    let Some(entries) = item.get("value").and_then(Value::as_array) else {
        return Err(ChainQueryError::Unexpected(
            "getDesignatedByRole did not return an array".to_string(),
        ));
    };
    entries
        .iter()
        .map(|entry| {
            entry
                .get("value")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ChainQueryError::Unexpected("a designated key is not a ByteString".to_string())
                })
                .and_then(decode_public_key)
        })
        .collect()
}

/// Decodes one base64 ByteString into the hex public key operators recognise.
fn decode_public_key(encoded: &str) -> Result<String, ChainQueryError> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let bytes = STANDARD.decode(encoded).map_err(|error| {
        ChainQueryError::Unexpected(format!("a designated key is not base64: {error}"))
    })?;
    if bytes.len() != 33 {
        return Err(ChainQueryError::Unexpected(format!(
            "a designated key is {} bytes, expected a 33-byte compressed point",
            bytes.len()
        )));
    }
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
#[path = "../../tests/unit/chain_state/designation/tests.rs"]
mod tests;
