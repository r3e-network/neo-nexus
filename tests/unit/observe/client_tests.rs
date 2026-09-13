use super::*;

/// A `-32601` means this client has no such method. It is a capability fact
/// and it must never read as an outage — conflating the two is what made a
/// healthy Neo X node, which has no `getversion`, report as unreachable.
#[test]
fn method_not_found_is_a_capability_fact_not_a_failure() {
    let body = r#"{"jsonrpc":"2.0","id":"x","error":{"code":-32601,"message":"Method not found"}}"#;
    let observed = read_result(body, "getversion", "http://127.0.0.1:8545", 1_770_000_000);
    assert_eq!(
        observed,
        Observation::Unknown(NotSampled::MethodUnsupported {
            method: "getversion"
        })
    );
    assert!(observed
        .render(|v: &serde_json::Value| v.to_string())
        .contains("does not implement"));
}

/// Any other JSON-RPC error is a failure, and carries what the node said.
#[test]
fn other_rpc_errors_are_failures_that_quote_the_node() {
    let body = r#"{"jsonrpc":"2.0","id":"x","error":{"code":-32602,"message":"Invalid params"}}"#;
    let observed = read_result(body, "getblockheader", "http://127.0.0.1:10332", 0);
    let Observation::Unknown(NotSampled::CallFailed { method, detail }) = &observed else {
        unreachable!("expected a call failure, got {observed:?}")
    };
    assert_eq!(*method, "getblockheader");
    assert!(detail.contains("Invalid params"), "{detail}");
    assert!(detail.contains("-32602"), "{detail}");
}

#[test]
fn a_result_is_kept_with_the_call_that_produced_it() {
    let body = r#"{"jsonrpc":"2.0","id":"x","result":6245100}"#;
    let observed = read_result(
        body,
        "getblockcount",
        "http://127.0.0.1:10332",
        1_770_000_000,
    );
    let evidence = observed.evidence().expect("a known value carries evidence");
    assert_eq!(evidence.method(), "getblockcount");
    assert_eq!(evidence.value(), "6245100");
    assert_eq!(evidence.endpoint(), "http://127.0.0.1:10332");
    assert_eq!(evidence.sampled_at_unix(), 1_770_000_000);
}

/// A reply that is neither a result nor an error is unusable, and says so —
/// rather than being treated as an empty success.
#[test]
fn a_reply_with_neither_result_nor_error_is_unusable() {
    let observed = read_result(r#"{"jsonrpc":"2.0","id":"x"}"#, "getblockcount", "", 0);
    assert!(matches!(
        observed,
        Observation::Unknown(NotSampled::CallFailed { .. })
    ));
}

#[test]
fn a_reply_that_is_not_json_is_reported_as_such() {
    let observed = read_result("<html>502 Bad Gateway</html>", "getblockcount", "", 0);
    let Observation::Unknown(NotSampled::CallFailed { detail, .. }) = &observed else {
        unreachable!("expected a call failure, got {observed:?}")
    };
    assert!(detail.contains("not JSON"), "{detail}");
}

/// Evidence is stored beside every sample, so an unbounded answer — a mempool
/// listing on a congested chain — is truncated rather than retained whole.
#[test]
fn evidence_of_a_large_answer_is_truncated_and_says_so() {
    let hashes: Vec<String> = (0..500).map(|n| format!("0x{n:064x}")).collect();
    let body = serde_json::json!({ "jsonrpc": "2.0", "id": "x", "result": hashes }).to_string();
    let observed = read_result(&body, "getrawmempool", "", 0);
    let evidence = observed.evidence().expect("known");
    assert!(
        evidence.value().len() < 400,
        "evidence was kept at full size ({} bytes)",
        evidence.value().len()
    );
    assert!(evidence.value().contains("bytes)"), "{}", evidence.value());
}

/// A short answer is kept exactly, because the point of evidence is to show
/// what the node said.
#[test]
fn evidence_of_a_short_answer_is_kept_verbatim() {
    let observed = read_result(r#"{"result":"0x1c2b3d"}"#, "eth_blockNumber", "", 0);
    assert_eq!(observed.evidence().map(Evidence::value), Some("0x1c2b3d"));
}
