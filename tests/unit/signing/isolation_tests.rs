use super::*;
use crate::signing::SignerKeyRef;

/// The refusal has to name the key and, in the operator's own vocabulary, the
/// instance already holding it. A raw node id identifies the offender in the one
/// spelling an operator does not recognise.
#[test]
fn signer_isolation_violation_displays_clear_messages() {
    let key = SignerKeyRef::new("wallet-profile", "consensus-key-01").unwrap();
    let err1 = SignerIsolationViolation::KeyAlreadyAssignedToOtherNode {
        key: key.clone(),
        owner: "validator-alpha".to_string(),
    };
    let msg1 = err1.to_string();
    assert!(msg1.contains("wallet-profile/consensus-key-01"));
    assert!(msg1.contains("validator-alpha"));
    assert!(msg1.contains("release it there first"));

    let err2 = SignerIsolationViolation::CrossNodeAccessDenied {
        key,
        owner: "validator-alpha".to_string(),
        caller: "validator-beta".to_string(),
    };
    let msg2 = err2.to_string();
    assert!(msg2.contains("validator-beta"));
    assert!(msg2.contains("validator-alpha"));
    assert!(msg2.contains("Access denied"));
}

/// Leases are compared by the whole key, not by its backend. Two keys in one
/// custody service are two leases.
#[test]
fn check_signer_binding_allows_a_different_key_in_the_same_backend() {
    let key = SignerKeyRef::new("wallet-profile", "key-01").unwrap();
    let existing = vec![(
        "node-1".to_string(),
        SignerKeyRef::new("wallet-profile", "key-02").unwrap(),
    )];
    assert!(check_signer_binding_allowed("node-2", &key, &existing, str::to_string).is_ok());
}

#[test]
fn check_signer_binding_allows_rebinding_to_same_node() {
    let key = SignerKeyRef::new("wallet-profile", "key-01").unwrap();
    let existing = vec![("node-1".to_string(), key.clone())];
    assert!(check_signer_binding_allowed("node-1", &key, &existing, str::to_string).is_ok());
}

#[test]
fn check_signer_binding_rejects_key_bound_to_different_node() {
    let key = SignerKeyRef::new("wallet-profile", "key-01").unwrap();
    let existing = vec![("node-1".to_string(), key.clone())];
    let err = check_signer_binding_allowed("node-2", &key, &existing, str::to_string).unwrap_err();
    assert_eq!(
        err,
        SignerIsolationViolation::KeyAlreadyAssignedToOtherNode {
            key,
            owner: "node-1".to_string(),
        }
    );
}

/// The caller supplies the naming, so the refusal can say "validator-alpha"
/// where the workspace knows the name and fall back to the id where it does not.
#[test]
fn the_refusal_names_the_owning_instance_the_way_the_caller_does() {
    let key = SignerKeyRef::new("wallet-profile", "key-01").unwrap();
    let existing = vec![("node-1".to_string(), key.clone())];
    let err = check_signer_binding_allowed("node-2", &key, &existing, |node_id| match node_id {
        "node-1" => "validator-alpha".to_string(),
        other => other.to_string(),
    })
    .unwrap_err();
    assert!(err.to_string().contains("validator-alpha"));
    assert!(!err.to_string().contains("node-1"));
}
