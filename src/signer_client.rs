//! Client-only integration with the NeoOS Rust signer.
//!
//! Private-key custody, policy decisions, anti-equivocation, and signer audit
//! storage deliberately do not exist in this crate.

mod auth;
mod client;
mod config;
mod model;

pub use auth::{
    body_sha256, workload_signing_message, workload_signing_message_v2, ApiKeyCredential,
    BearerCredential, OidcCredential, SignerCredential, WorkloadCredential,
};
pub use client::{SignerClient, SignerClientError, SignerClientErrorKind};
pub use config::{SignerClientConfig, SignerEndpoint};
pub use model::*;

pub(crate) use auth::AuthHeaders;
