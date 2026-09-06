//! HTTP client for the remote Rust signer service.
//!
//! The public [`SignerClient`] is deliberately small. Endpoint methods are
//! grouped by the signer contract's two trust surfaces:
//!
//! * [`signing`] carries a caller's own credential to the §5 signing routes;
//! * [`admin`] carries an explicitly supplied admin credential to §5.1;
//! * [`authentication`] separates configured admin proof generation from the
//!   public relay's strict header allowlist; and
//! * [`transport`] owns the HTTP/JSON implementation and its response-envelope,
//!   identifier, and diagnostic rules.
//!
//! Keeping those responsibilities separate makes the custody boundary visible
//! in the source tree without changing the synchronous API used by the web
//! relay. Axum handlers still call this blocking client through
//! `tokio::task::spawn_blocking`.

use std::{fmt, sync::Arc};

use super::SignerConfig;

mod admin;
mod authentication;
mod signing;
mod transport;

pub(crate) use authentication::ForwardedCredentials;
pub(crate) use transport::MAX_REQUEST_BODY_BYTES;

/// A validated signer configuration and one pooled, redirect-free HTTP agent.
#[derive(Clone)]
pub struct SignerClient {
    config: Arc<SignerConfig>,
    agent: ureq::Agent,
}

impl fmt::Debug for SignerClient {
    /// Prints the destination and timeout, never a credential.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerClient")
            .field("base_url", &self.config.base_url())
            .field("timeout", &self.config.timeout())
            .field("admin_configured", &self.config.admin().is_some())
            .finish()
    }
}

impl SignerClient {
    pub fn new(config: SignerConfig) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(config.timeout())
            // A redirect is a different custody endpoint. Following one could
            // deliver a bearer credential or unsigned transaction to a host the
            // operator did not configure.
            .redirects(0)
            .build();
        Self {
            config: Arc::new(config),
            agent,
        }
    }

    pub fn config(&self) -> &SignerConfig {
        &self.config
    }
}

// The existing unit module tests these boundary helpers directly. Keeping the
// names at this facade also documents which transport decisions are important
// enough to test independent of an HTTP server.
#[cfg(test)]
use transport::{checked_id, parse_reply, read_limited_body, to_json, MAX_RESPONSE_BODY_BYTES};

#[cfg(test)]
#[path = "../../tests/unit/signer_client/client/tests.rs"]
mod tests;
