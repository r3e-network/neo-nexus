//! Rust representation of the remote signer HTTP contract.
//!
//! The vocabulary is grouped by responsibility while this facade preserves the
//! original flat public API:
//!
//! * [`decision`] contains the common response envelope and caller credential;
//! * [`signing`] mirrors §5 signing requests and successful responses; and
//! * [`admin`] mirrors §5.1 keys, policy, callers, and audit records.
//!
//! These are transport models, not a second policy or cryptography engine.
//! Unknown additive response fields remain preserved at the relay boundary,
//! refusal codes remain open strings, and token-bearing types redact `Debug`.

mod admin;
mod decision;
mod signing;

pub use admin::{
    AssetLimit, AuditRow, Caller, ContractMethod, CreatedCaller, CreatedWorkloadCaller, Grant,
    KeyBoundary, KeyPublic, Policy, PolicyAdvice, RemovedCaller, RemovedKey, RotatedCaller,
    SavedBoundary, SignatureRateLimit, WindowLimit, WorkloadCallerRequest,
};
pub(crate) use decision::WorkloadCredential;
pub use decision::{CallerToken, Outcome, Refusal};
pub use signing::{
    Eip191Fulfillment, Eip191FulfillmentRequest, Eip191FulfillmentSignature, GenerateKeyRequest,
    RawSignRequest, RawSignature, SignRequest, Signature,
};

#[cfg(test)]
#[path = "../../tests/unit/signer_client/wire/tests.rs"]
mod tests;
