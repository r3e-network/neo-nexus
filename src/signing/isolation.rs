//! Strict IAM Instance Profile & Key Isolation Boundary.
//!
//! A signer key or wallet is leased exclusively to at most one node instance
//! (acting as an IAM Instance Profile). Cross-node access, key usurpation,
//! and dual-consensus signing are strictly forbidden by policy.

use std::fmt;

use crate::signing::SignerKeyRef;

#[derive(Debug, PartialEq, Eq)]
pub enum SignerIsolationViolation {
    KeyAlreadyAssignedToOtherNode {
        key: SignerKeyRef,
        owner_node_id: String,
    },
    CrossNodeAccessDenied {
        key: SignerKeyRef,
        owner_node_id: String,
        caller_node_id: String,
    },
}

impl fmt::Display for SignerIsolationViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyAlreadyAssignedToOtherNode { key, owner_node_id } => {
                write!(
                    f,
                    "Signer key '{}/{}' is already exclusively allocated to instance '{owner_node_id}'. Cross-node key usage is strictly forbidden by IAM isolation policy.",
                    key.backend_id, key.key_id
                )
            }
            Self::CrossNodeAccessDenied {
                key,
                owner_node_id,
                caller_node_id,
            } => {
                write!(
                    f,
                    "Instance '{caller_node_id}' attempted to sign with key '{}/{}' owned exclusively by '{owner_node_id}'. Access denied.",
                    key.backend_id, key.key_id
                )
            }
        }
    }
}

impl std::error::Error for SignerIsolationViolation {}

/// Enforce that a key is not already bound to another node.
pub fn check_signer_binding_allowed(
    target_node_id: &str,
    key: &SignerKeyRef,
    existing_bindings: &[(String, SignerKeyRef)],
) -> Result<(), SignerIsolationViolation> {
    for (node_id, bound_key) in existing_bindings {
        if bound_key == key && node_id != target_node_id {
            return Err(SignerIsolationViolation::KeyAlreadyAssignedToOtherNode {
                key: key.clone(),
                owner_node_id: node_id.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/signing/isolation_tests.rs"]
mod tests;
