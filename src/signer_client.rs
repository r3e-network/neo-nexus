//! The service-client half of the custody boundary: this workbench reaches a
//! signer over the HTTP contract in `neo-os-services/docs/SIGNER_SERVICE.md`
//! §5 and §5.1. This module holds no Neo custody/signing key material.
//!
//! That last clause is the reason this module exists. The service-side policy
//! engine used to be copied into neo-nexus — a vault sealed by a master key in
//! this process's environment and five tables in `neonexus.db`. Three copies of
//! that arrangement drifted apart across two repositories and a Node.js service.
//! The shared service engine now lives in `neo-os-services/workers/neo-signer`,
//! and what lives here is a transport. The ownership matrix in §3 is the
//! authority for either service-backed mode.
//!
//! This scope is deliberately narrower than the crate's signing subsystem. The
//! separately selected `crate::signing::LocalWalletSigner` is a process-local,
//! Neo N3-only capability; it never passes through this client, the public relay,
//! or the service administration surface.
//!
//! ## What this module may not grow
//!
//! No custody cryptography, sealing, signing-key storage, or `signer_*` rows. The
//! sole private material allowed here is NeoNexus's own Ed25519 workload seed,
//! used only to authenticate an admin HTTP request; it can never produce a Neo
//! transaction signature. If a future change reaches for `p256`, `aes_gcm`,
//! `scrypt`, WIF/NEP-2 parsing, or `crate::repository`, the migration in §7 step
//! 2 has not finished — it has just been renamed.
//!
//! The v1 service has key-import routes, but this consumer deliberately does not.
//! Accepting a WIF, raw private key, NEP-2 value or passphrase would make the
//! control plane a secret ingress even if it promised not to persist the bytes.
//! Import belongs at the signer service's trusted operator boundary until v2
//! attestation lets a remote console prove which enclave and vault receive it.
//!
//! ## The two consumers
//!
//! * The client's §5 relay serves `src/web/signer_api.rs`, which authenticates
//!   nobody and decides nothing: it forwards only a caller's bearer/workload
//!   authentication headers plus the exact raw body bytes, then relays whatever
//!   the service answered, status included.
//! * Its §5.1 methods serve the operator console, which presents the admin
//!   identity from a protected bearer-token file or a protected Ed25519 workload
//!   seed file, and therefore can configure a key it cannot use — the split §5.1
//!   was written to make possible.

mod client;
mod config;
mod wire;

pub use client::SignerClient;
pub(crate) use client::{ForwardedCredentials, MAX_REQUEST_BODY_BYTES};
pub(crate) use config::SignerProfileInput;
pub use config::{
    SignerConfig, API_PREFIX, CALLER_ID_ENV, DEFAULT_TIMEOUT, LEGACY_TOKEN_ENV, LEGACY_URL_ENV,
    ORIGIN_ENV, TIMEOUT_ENV, TOKEN_ENV, TOKEN_FILE_ENV, URL_ENV, WORKLOAD_KEY_FILE_ENV,
    WORKLOAD_SUBJECT_ENV,
};
pub use wire::{
    AssetLimit, AuditRow, Caller, CallerToken, ContractMethod, CreatedCaller,
    CreatedWorkloadCaller, Eip191Fulfillment, Eip191FulfillmentRequest, Eip191FulfillmentSignature,
    GenerateKeyRequest, Grant, KeyBoundary, KeyPublic, Outcome, Policy, PolicyAdvice,
    RawSignRequest, RawSignature, Refusal, RemovedCaller, RemovedKey, RotatedCaller, SavedBoundary,
    SignRequest, Signature, SignatureRateLimit, WindowLimit, WorkloadCallerRequest,
};
