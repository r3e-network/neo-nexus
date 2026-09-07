//! Tests for committee roster parsing and handoff summaries

use crate::private_network::CommitteeRoster;

fn key(prefix: &str, fill: char) -> String {
    format!("{prefix}{}", fill.to_string().repeat(64))
}

#[test]
fn from_public_keys_returns_none_for_empty_input() {
    assert!(CommitteeRoster::from_public_keys("   ")
        .expect("empty input is not an error")
        .is_none());
}

#[test]
fn from_public_keys_parses_and_labels_multiple_keys() {
    let input = format!(
        "{}, {} ; {}",
        key("02", 'a'),
        key("03", 'b'),
        key("02", 'c')
    );
    let roster = CommitteeRoster::from_public_keys(&input)
        .expect("valid keys parse")
        .expect("non-empty roster");

    assert_eq!(roster.signers.len(), 3);
    assert_eq!(roster.signers[0].label, "committee-signer-1");
    assert_eq!(roster.signers[2].label, "committee-signer-3");
    assert_eq!(roster.public_keys(), roster.public_keys());
    assert_eq!(roster.public_keys()[0], key("02", 'a'));
}

#[test]
fn from_public_keys_rejects_duplicate_keys() {
    let duplicate = key("03", 'a');
    let input = format!("{duplicate} {duplicate}");
    let error = CommitteeRoster::from_public_keys(&input).expect_err("duplicates must fail");
    assert!(error.to_string().contains("duplicate committee public key"));
}

#[test]
fn from_public_keys_rejects_malformed_keys() {
    assert!(CommitteeRoster::from_public_keys("deadbeef").is_err());
    assert!(CommitteeRoster::from_public_keys(&key("04", 'a')).is_err());
    assert!(CommitteeRoster::from_public_keys(&format!("02{}", "z".repeat(64))).is_err());
}

#[test]
fn handoff_summary_reports_missing_signers_when_below_required() {
    let roster = CommitteeRoster::from_public_keys(&key("02", 'a'))
        .unwrap()
        .unwrap();
    let summary = roster.handoff_summary(3);
    assert_eq!(summary.signer_count, 1);
    assert_eq!(summary.required_signer_count, 3);
    assert_eq!(summary.missing_required_signer_count, 2);
    assert_eq!(summary.status_label(), "incomplete");
    assert!(summary.operator_summary().contains("incomplete"));
}

#[test]
fn handoff_summary_flags_wallets_pending_when_all_signers_present() {
    let input = format!("{} {}", key("02", 'a'), key("03", 'b'));
    let roster = CommitteeRoster::from_public_keys(&input).unwrap().unwrap();
    let summary = roster.handoff_summary(2);
    assert_eq!(summary.missing_required_signer_count, 0);
    assert_eq!(summary.missing_wallet_reference_count, 2);
    assert_eq!(summary.status_label(), "wallets pending");
}
