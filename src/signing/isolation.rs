//! The exclusive-lease rule for signer identities.
//!
//! A signer key is leased to at most one node, the way a cloud instance profile
//! belongs to one instance. Cross-node use, key usurpation, and two nodes
//! signing consensus with one key are refused.
//!
//! This module owns the *wording* of that refusal as well as the check. The
//! rule is enforced in three places — the instance editor, the instance detail
//! page, and the repository write itself — and until they shared this text they
//! told the operator three different things about the same refusal.

use std::fmt;

use crate::signing::SignerKeyRef;

#[derive(Debug, PartialEq, Eq)]
pub enum SignerIsolationViolation {
    KeyAlreadyAssignedToOtherNode {
        key: SignerKeyRef,
        /// The instance holding the lease, named the way an operator sees it.
        owner: String,
    },
    CrossNodeAccessDenied {
        key: SignerKeyRef,
        owner: String,
        caller: String,
    },
}

impl fmt::Display for SignerIsolationViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyAlreadyAssignedToOtherNode { key, owner } => {
                write!(
                    f,
                    "Signer key '{}/{}' is already leased exclusively to instance '{owner}'. One key signs for one instance; release it there first.",
                    key.backend_id, key.key_id
                )
            }
            Self::CrossNodeAccessDenied { key, owner, caller } => {
                write!(
                    f,
                    "Instance '{caller}' attempted to sign with key '{}/{}', which is leased exclusively to '{owner}'. Access denied.",
                    key.backend_id, key.key_id
                )
            }
        }
    }
}

impl std::error::Error for SignerIsolationViolation {}

/// Enforce that a key is not already leased to another node.
///
/// `existing_leases` is every recorded lease as `(node id, key)`. `naming`
/// turns a node id into whatever the caller shows an operator — a name where
/// one is known, the id where it is not. Before this, the refusal quoted a raw
/// uuid, which named the offending instance in the one vocabulary an operator
/// does not have.
pub fn check_signer_binding_allowed(
    target_node_id: &str,
    key: &SignerKeyRef,
    existing_leases: &[(String, SignerKeyRef)],
    naming: impl Fn(&str) -> String,
) -> Result<(), SignerIsolationViolation> {
    for (node_id, leased) in existing_leases {
        if leased == key && node_id != target_node_id {
            return Err(SignerIsolationViolation::KeyAlreadyAssignedToOtherNode {
                key: key.clone(),
                owner: naming(node_id),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/signing/isolation_tests.rs"]
mod tests;
