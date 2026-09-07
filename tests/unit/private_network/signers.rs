//! Tests for signer endpoint, template and command-plan validation helpers

use crate::private_network::{
    expand_signer_command_template, has_signer_references, parse_signer_command_plan,
    signer_command_plan_matches_command, signer_endpoint_is_remote_cleartext,
    validate_signer_command_template, validate_signer_endpoint, CommitteeSigner,
};

#[test]
fn validate_signer_endpoint_accepts_http_and_https_urls() {
    assert_eq!(
        validate_signer_endpoint("https://signer.example.com:9000").unwrap(),
        "https://signer.example.com:9000"
    );
    assert!(validate_signer_endpoint("http://127.0.0.1:9000").is_ok());
}

#[test]
fn validate_signer_endpoint_rejects_bad_schemes_and_credentials() {
    assert!(validate_signer_endpoint("ftp://signer.example.com").is_err());
    assert!(validate_signer_endpoint("https://user:pass@signer.example.com").is_err());
    assert!(validate_signer_endpoint("https://signer.example.com/#frag").is_err());
}

#[test]
fn signer_endpoint_is_remote_cleartext_flags_non_loopback_http() {
    assert!(signer_endpoint_is_remote_cleartext(
        "http://signer.example.com:9000"
    ));
    assert!(!signer_endpoint_is_remote_cleartext(
        "http://127.0.0.1:9000"
    ));
    assert!(!signer_endpoint_is_remote_cleartext(
        "http://localhost:9000"
    ));
    assert!(!signer_endpoint_is_remote_cleartext(
        "https://signer.example.com"
    ));
}

#[test]
fn has_signer_references_detects_non_blank_lines() {
    assert!(has_signer_references("committee-signer-1 wallet=w.json"));
    assert!(!has_signer_references("   \n\n"));
}

#[test]
fn validate_signer_command_template_rejects_unknown_placeholders() {
    assert!(validate_signer_command_template("run --wallet {wallet}").is_ok());
    assert!(validate_signer_command_template("run --secret {password}").is_err());
    assert!(validate_signer_command_template("run --wallet {wallet").is_err());
}

#[test]
fn expand_signer_command_template_substitutes_signer_fields() {
    let signer = CommitteeSigner {
        label: "committee-signer-1".to_string(),
        public_key: "03abc".to_string(),
        wallet_path: Some("wallets/c1.json".into()),
        signer_endpoint: Some("https://signer.example.com".to_string()),
        signer_command_template: None,
        signer_command: None,
        signer_command_plan: None,
    };
    let expanded =
        expand_signer_command_template("neo-signer --wallet {wallet} --label {label}", &signer)
            .expect("template expands");
    assert_eq!(
        expanded,
        "neo-signer --wallet wallets/c1.json --label committee-signer-1"
    );
}

#[test]
fn parse_signer_command_plan_splits_tokens_and_sets_policy() {
    let plan = parse_signer_command_plan("neo-signer --port 20333").expect("valid command");
    assert_eq!(plan.execution_policy, "argv-no-shell");
    assert_eq!(plan.binary, "neo-signer");
    assert_eq!(plan.arguments, vec!["--port", "20333"]);
    assert!(signer_command_plan_matches_command(
        &plan,
        "neo-signer --port 20333"
    ));
}

#[test]
fn parse_signer_command_plan_rejects_pipes_and_empty_commands() {
    assert!(parse_signer_command_plan("").is_err());
    assert!(parse_signer_command_plan("neo-signer | tee log").is_err());
    assert!(parse_signer_command_plan("neo-signer 'unclosed").is_err());
}
