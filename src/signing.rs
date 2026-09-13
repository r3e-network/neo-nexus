//! Typed signing backends used by the node workbench.
//!
//! A local encrypted wallet, a signer process on this host, and the NeoOS
//! signer service have different trust and lifecycle properties.  They share
//! a vocabulary, not an implementation: only service backends have a caller
//! registry, centrally enforced policy, a durable audit trail, and a public
//! relay.  Keeping those capabilities explicit prevents a local wallet from
//! becoming an accidental fallback when a service is unavailable.

mod capability;
mod environment;
mod isolation;
mod local_signer;
mod local_wallet;
mod profile;
mod registry;

pub use isolation::{check_signer_binding_allowed, SignerIsolationViolation};

pub use capability::SignerCapabilities;
pub use local_signer::{
    LocalSignerConfig, LOCAL_SIGNER_ENDPOINT_ENV, LOCAL_SIGNER_NETWORK_MAGIC_ENV,
    LOCAL_SIGNER_PUBLIC_KEY_ENV,
};
pub use local_wallet::{
    LocalWalletConfig, LocalWalletEnvironment, LocalWalletSigner, LOCAL_WALLET_ACCOUNT_ENV,
    LOCAL_WALLET_ALLOW_CONSENSUS_ENV, LOCAL_WALLET_ALLOW_RAW_ENV,
    LOCAL_WALLET_ALLOW_TRANSACTION_ENV, LOCAL_WALLET_NETWORK_ENV, LOCAL_WALLET_NETWORK_MAGIC_ENV,
    LOCAL_WALLET_PASSWORD_FILE_ENV, LOCAL_WALLET_PATH_ENV,
};
pub use profile::{SignerBackendKind, SignerBackendProfile, SignerKeyRef, BACKEND_ENV};
pub use registry::{
    ConfiguredSignerBackend, ServiceSignerBackend, SignerRegistry, PROFILES_FILE_ENV,
};

#[cfg(test)]
#[path = "../tests/unit/signing/tests.rs"]
mod tests;
