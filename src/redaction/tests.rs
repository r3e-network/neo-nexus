use super::*;

#[test]
fn redacts_sensitive_argv_shapes() {
    let redacted = redact_sensitive_args(&[
        "--api-key:abc123".to_string(),
        "--mnemonic".to_string(),
        "twelve secret words".to_string(),
        "--authorization".to_string(),
        "Bearer".to_string(),
        "node-access-token".to_string(),
        "--public-rpc".to_string(),
        "http://127.0.0.1:10332".to_string(),
        "--seed=raw-seed".to_string(),
        "--wallet_key".to_string(),
        "wallet-private".to_string(),
    ]);

    assert_eq!(
        redacted,
        [
            "--api-key:<redacted>",
            "--mnemonic",
            REDACTED_VALUE,
            "--authorization",
            REDACTED_VALUE,
            REDACTED_VALUE,
            "--public-rpc",
            "http://127.0.0.1:10332",
            "--seed=<redacted>",
            "--wallet_key",
            REDACTED_VALUE,
        ]
    );
}

#[test]
fn redacts_sensitive_text_shapes() {
    let redacted = redact_sensitive_text(
        "Authorization: Bearer node-access-token api_key:abc123 seed=raw-seed webhook=https://hooks.example/token",
    );

    assert!(redacted.contains("Authorization:<redacted>"));
    assert!(redacted.contains("api_key:<redacted>"));
    assert!(redacted.contains("seed=<redacted>"));
    assert!(redacted.contains("webhook=<redacted>"));
    assert!(!redacted.contains("node-access-token"));
    assert!(!redacted.contains("abc123"));
    assert!(!redacted.contains("raw-seed"));
    assert!(!redacted.contains("hooks.example"));
}

#[test]
fn redacts_sensitive_query_values_after_public_parameters() {
    let redacted = redact_sensitive_text(
        "callback=https://node.example/cb?network=testnet&access_token=abc123&height=42 jwt=header.payload.signature",
    );

    assert!(redacted.contains("network=testnet"));
    assert!(redacted.contains("access_token=<redacted>"));
    assert!(redacted.contains("height=42"));
    assert!(redacted.contains("jwt=<redacted>"));
    assert!(!redacted.contains("abc123"));
    assert!(!redacted.contains("header.payload.signature"));
}

#[test]
fn redacts_sensitive_query_values_in_args() {
    let redacted = redact_sensitive_args(&[
        "--rpc-url".to_string(),
        "https://node.example/rpc?network=mainnet&token=abc123&height=99".to_string(),
        "--cookie".to_string(),
        "session-cookie".to_string(),
    ]);

    assert_eq!(
        redacted,
        [
            "--rpc-url",
            "https://node.example/rpc?network=mainnet&token=<redacted>&height=99",
            "--cookie",
            REDACTED_VALUE,
        ]
    );
}

#[test]
fn redacts_alert_provider_secret_keys() {
    let redacted = redact_sensitive_text(
        r#"{"routing_key":"pager-key","chat_id":"-100123"} https://events.pagerduty.com/v2/enqueue?routing_key=pager-key&dedup_key=node-1"#,
    );

    assert!(redacted.contains("routing_key"));
    assert!(redacted.contains("chat_id"));
    assert!(redacted.contains(REDACTED_VALUE));
    assert!(redacted.contains("dedup_key=node-1"));
    assert!(!redacted.contains("pager-key"));
    assert!(!redacted.contains("-100123"));
}
