use super::*;
use crate::signing::SignerKeyRef;

#[test]
fn signer_isolation_violation_displays_clear_messages() {
    let key = SignerKeyRef::new("wallet-profile", "consensus-key-01").unwrap();
    let err1 = SignerIsolationViolation::KeyAlreadyAssignedToOtherNode {
        key: key.clone(),
        owner_node_id: "node-alpha".to_string(),
    };
    let msg1 = err1.to_string();
    assert!(msg1.contains("wallet-profile/consensus-key-01"));
    assert!(msg1.contains("node-alpha"));
    assert!(msg1.contains("IAM isolation policy"));

    let err2 = SignerIsolationViolation::CrossNodeAccessDenied {
        key,
        owner_node_id: "node-alpha".to_string(),
        caller_node_id: "node-beta".to_string(),
    };
    let msg2 = err2.to_string();
    assert!(msg2.contains("node-beta"));
    assert!(msg2.contains("node-alpha"));
    assert!(msg2.contains("Access denied"));
}

#[test]
fn check_signer_binding_allows_unbound_key() {
    let key = SignerKeyRef::new("wallet-profile", "key-01").unwrap();
    let existing = vec![(
        "node-1".to_string(),
        SignerKeyRef::new("wallet-profile", "key-02").unwrap(),
    )];
    assert!(check_signer_binding_allowed("node-2", &key, &existing).is_ok());
}

#[test]
fn check_signer_binding_allows_rebinding_to_same_node() {
    let key = SignerKeyRef::new("wallet-profile", "key-01").unwrap();
    let existing = vec![("node-1".to_string(), key.clone())];
    assert!(check_signer_binding_allowed("node-1", &key, &existing).is_ok());
}

#[test]
fn check_signer_binding_rejects_key_bound_to_different_node() {
    let key = SignerKeyRef::new("wallet-profile", "key-01").unwrap();
    let existing = vec![("node-1".to_string(), key.clone())];
    let err = check_signer_binding_allowed("node-2", &key, &existing).unwrap_err();
    assert_eq!(
        err,
        SignerIsolationViolation::KeyAlreadyAssignedToOtherNode {
            key,
            owner_node_id: "node-1".to_string(),
        }
    );
}
