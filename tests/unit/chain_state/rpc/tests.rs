use super::{invocation_stack, parse_result};
use crate::chain_state::ChainQueryError;

#[test]
fn a_result_envelope_is_unwrapped() {
    let value = parse_result(
        "getblockcount",
        r#"{"jsonrpc":"2.0","id":1,"result":8123456}"#,
    )
    .expect("a result should parse");
    assert_eq!(value.as_u64(), Some(8_123_456));
}

/// A node that answers with a JSON-RPC error is reachable — the request was
/// wrong or unsupported, which is a different problem from a dead endpoint.
#[test]
fn a_json_rpc_error_is_unexpected_not_unreachable() {
    let error = parse_result(
        "invokefunction",
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#,
    )
    .expect_err("an error envelope must not parse as success");
    assert!(matches!(error, ChainQueryError::Unexpected(_)));
    assert!(error.message().contains("Method not found"));
}

#[test]
fn a_missing_result_is_reported_rather_than_defaulted() {
    let error = parse_result("getcommittee", r#"{"jsonrpc":"2.0","id":1}"#)
        .expect_err("an envelope with no result is not success");
    assert!(error.message().contains("no result"));
}

#[test]
fn non_json_is_reported_as_unexpected() {
    let error = parse_result("getversion", "<html>502 Bad Gateway</html>")
        .expect_err("HTML is not a JSON-RPC response");
    assert!(matches!(error, ChainQueryError::Unexpected(_)));
}

/// A FAULTed contract call returns HTTP 200 with a well-formed envelope. Taking
/// its stack at face value would read a failed invocation as an empty answer —
/// reporting "nobody holds this role" when the truth is "the call did not run".
#[test]
fn a_faulted_invocation_is_not_mistaken_for_an_empty_answer() {
    let result = serde_json::json!({
        "state": "FAULT",
        "exception": "Specified argument was out of the range of valid values.",
        "stack": []
    });
    let error = invocation_stack("getDesignatedByRole", &result)
        .expect_err("a FAULT must not be read as a result");
    assert!(error.message().contains("FAULT"));
    assert!(error.message().contains("out of the range"));
}

#[test]
fn a_halted_invocation_yields_its_first_stack_item() {
    let result = serde_json::json!({
        "state": "HALT",
        "stack": [{ "type": "Array", "value": [] }]
    });
    let item = invocation_stack("getDesignatedByRole", &result).expect("HALT yields a stack");
    assert_eq!(item["type"], "Array");
}
