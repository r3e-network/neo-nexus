//! Named signer profiles and explicit, backend-qualified dispatch.
//!
//! A registry may hold node-local wallets, native `SecureSign` gRPC deployments,
//! and NeoOS HTTP custody services at the same time. Those protocols are never
//! inferred from endpoint location or routed through one another.

mod backend;
mod catalog;
mod document;

pub use backend::{ConfiguredSignerBackend, ServiceSignerBackend};
pub use catalog::SignerRegistry;

pub const PROFILES_FILE_ENV: &str = "NEONEXUS_SIGNER_PROFILES_FILE";

#[cfg(test)]
mod response_tests {
    use super::*;
    use crate::signer_client::Outcome;
    use crate::signing::SignerKeyRef;
    use catalog::ensure_outcome_key;

    #[test]
    fn allowed_service_response_must_name_the_bound_key() {
        let key = SignerKeyRef {
            backend_id: "service".to_string(),
            key_id: "expected-key".to_string(),
        };
        let matching = Outcome::Allowed("expected-key".to_string());
        let mismatched = Outcome::Allowed("other-key".to_string());

        assert!(ensure_outcome_key(&key, &matching, String::as_str).is_ok());
        assert!(ensure_outcome_key(&key, &mismatched, String::as_str).is_err());
    }
}
