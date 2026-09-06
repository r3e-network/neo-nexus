//! Validated signer endpoint and operator credential configuration.
//!
//! Production acquisition is intentionally narrower than the programmatic
//! constructors kept for compatibility. [`SignerConfig::from_env`] accepts the
//! native signer origin and exactly one protected, file-backed admin profile;
//! bearer values in process environments are rejected. Existing callers can
//! still construct a relay-only or test client directly.

use std::{
    fmt::{self, Debug},
    path::PathBuf,
    time::Duration,
};

#[cfg(test)]
use anyhow::Context;
use anyhow::{bail, Result};
use zeroize::{Zeroize, Zeroizing};

use super::{wire::WorkloadCredential, CallerToken};

mod environment;
mod secret_file;
mod validation;

use environment::configured;
use validation::{
    validated_compatible_base_url, validated_consumer_origin, validated_service_url,
    validated_workload_identity,
};

/// Canonical signer service origin used by production configuration.
pub const URL_ENV: &str = "NEONEXUS_SIGNER_URL";
/// One-release compatibility alias for [`URL_ENV`]. Setting both is an error.
pub const LEGACY_URL_ENV: &str = "NEONEXUS_SIGNER_SERVICE_URL";
/// Protected file containing a bearer admin token.
pub const TOKEN_FILE_ENV: &str = "NEONEXUS_SIGNER_ADMIN_TOKEN_FILE";
/// Admin workload caller id paired with [`WORKLOAD_KEY_FILE_ENV`].
pub const CALLER_ID_ENV: &str = "NEONEXUS_SIGNER_ADMIN_CALLER_ID";
/// Protected file containing one lowercase-hex 32-byte Ed25519 seed.
pub const WORKLOAD_KEY_FILE_ENV: &str = "NEONEXUS_SIGNER_ADMIN_WORKLOAD_KEY_FILE";
/// Optional subject pinned on the admin workload caller.
pub const WORKLOAD_SUBJECT_ENV: &str = "NEONEXUS_SIGNER_ADMIN_WORKLOAD_SUBJECT";

/// Rejected legacy plaintext secret input. Kept as a public constant so
/// existing integrations receive a precise migration diagnostic.
pub const LEGACY_TOKEN_ENV: &str = "NEONEXUS_SIGNER_SERVICE_TOKEN";
/// Backwards-compatible name for [`LEGACY_TOKEN_ENV`].
pub const TOKEN_ENV: &str = LEGACY_TOKEN_ENV;

/// Optional exact Origin for compatibility bearer callers. It is never applied
/// to a workload profile or to relayed signing callers.
pub const ORIGIN_ENV: &str = "NEONEXUS_SIGNER_SERVICE_ORIGIN";
/// Per-request timeout in whole seconds.
pub const TIMEOUT_ENV: &str = "NEONEXUS_SIGNER_SERVICE_TIMEOUT_SECONDS";
/// Prefix owned jointly by the Rust service and this client.
pub const API_PREFIX: &str = "/signer/api/v1";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// File-backed credential values supplied by a named signer profile.
///
/// This stays crate-private because callers should configure it through the
/// signer registry document rather than construct secret-bearing transports
/// ad hoc. It deliberately contains paths, never secret values.
pub(crate) struct SignerProfileInput {
    pub service_url: String,
    pub token_file: Option<PathBuf>,
    pub caller_id: Option<String>,
    pub workload_key_file: Option<PathBuf>,
    pub workload_subject: Option<String>,
    pub consumer_origin: Option<String>,
    pub timeout: Duration,
}

#[derive(Clone)]
enum AdminCredential {
    Bearer(Zeroizing<String>),
    Workload(Box<WorkloadCredential>),
}

/// Resolved endpoint and optional operator credential.
#[derive(Clone)]
pub struct SignerConfig {
    origin: String,
    // Non-empty only for compatibility constructors. Production `from_env`
    // talks to the native root contract and always leaves this empty.
    mount_path: String,
    admin: Option<AdminCredential>,
    consumer_origin: Option<String>,
    timeout: Duration,
    cleartext: bool,
    loopback: bool,
    // Plain HTTP exists only for deterministic in-process test servers. Every
    // production/profile/environment path leaves this false, and transport
    // refuses to put credentials or signing material on cleartext otherwise.
    allow_insecure_loopback_http_for_test: bool,
}

impl Debug for SignerConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let admin_mode = match self.admin {
            Some(AdminCredential::Bearer(_)) => "bearer-file:<redacted>",
            Some(AdminCredential::Workload(_)) => "workload-ed25519:<redacted>",
            None => "<none>",
        };
        formatter
            .debug_struct("SignerConfig")
            .field("origin", &self.origin)
            .field("mount_path", &self.mount_path)
            .field("admin", &admin_mode)
            .field("consumer_origin", &self.consumer_origin)
            .field("timeout", &self.timeout)
            .field("cleartext", &self.cleartext)
            .field("loopback", &self.loopback)
            .field(
                "insecure_test_transport",
                &self.allow_insecure_loopback_http_for_test,
            )
            .finish()
    }
}

impl SignerConfig {
    /// Build one native service transport from a named, file-backed profile.
    pub(crate) fn from_profile(input: SignerProfileInput) -> Result<Self> {
        let timeout = input.timeout.as_secs();
        let resolved = Self::resolve_environment(environment::EnvironmentInput {
            canonical_url: Some(input.service_url),
            legacy_url: None,
            legacy_token: None,
            token_file: input
                .token_file
                .map(|path| path.to_string_lossy().into_owned()),
            caller_id: input.caller_id,
            workload_key_file: input
                .workload_key_file
                .map(|path| path.to_string_lossy().into_owned()),
            workload_subject: input.workload_subject,
            consumer_origin: input.consumer_origin,
            timeout: Some(timeout.to_string()),
        })?;
        resolved.ok_or_else(|| anyhow::anyhow!("the signer service profile is empty"))
    }

    /// Compatibility resolver used by existing unit and embedding callers.
    ///
    /// Production startup does not call this function: it uses file-backed
    /// profiles through [`SignerConfig::from_env`].
    #[cfg(test)]
    pub(crate) fn resolve(
        raw_url: Option<String>,
        raw_token: Option<String>,
        raw_origin: Option<String>,
        raw_timeout: Option<String>,
    ) -> Result<Option<Self>> {
        let raw_url = configured(raw_url);
        let token = configured(raw_token);
        let consumer_origin = configured(raw_origin);
        let raw_timeout = configured(raw_timeout);
        if raw_url.is_none()
            && token.is_none()
            && consumer_origin.is_none()
            && raw_timeout.is_none()
        {
            return Ok(None);
        }
        let Some(raw_url) = raw_url else {
            bail!("{URL_ENV} must be set when compatibility signer settings are used");
        };
        let Some(token) = token else {
            bail!("{LEGACY_TOKEN_ENV} must be set for compatibility resolution");
        };
        let consumer_origin = consumer_origin
            .map(|origin| {
                validated_consumer_origin(&origin)
                    .with_context(|| format!("{ORIGIN_ENV} is not a usable browser Origin"))
            })
            .transpose()?;
        let timeout = environment::parse_timeout(raw_timeout)?;
        Ok(Some(
            Self::new_with_origin(&raw_url, Some(token), consumer_origin, timeout).with_context(
                || format!("{URL_ENV} is not a usable signer service configuration"),
            )?,
        ))
    }

    /// Build a bearer or relay-only client directly.
    pub fn new(base_url: &str, admin_token: Option<String>, timeout: Duration) -> Result<Self> {
        Self::new_with_origin(base_url, admin_token, None, timeout)
    }

    /// Compatibility constructor supporting an explicit reverse-proxy mount
    /// path and browser Origin. Production environment configuration is stricter.
    pub fn new_with_origin(
        base_url: &str,
        admin_token: Option<String>,
        consumer_origin: Option<String>,
        timeout: Duration,
    ) -> Result<Self> {
        let (origin, mount_path, cleartext, loopback) = validated_compatible_base_url(base_url)?;
        let consumer_origin = configured(consumer_origin)
            .map(|value| validated_consumer_origin(&value))
            .transpose()?;
        Ok(Self {
            origin,
            mount_path,
            admin: configured(admin_token)
                .map(|token| AdminCredential::Bearer(Zeroizing::new(token))),
            consumer_origin,
            timeout,
            cleartext,
            loopback,
            allow_insecure_loopback_http_for_test: false,
        })
    }

    /// Build a client for a loopback HTTP stub owned by the current test.
    ///
    /// This is deliberately loud and hidden from generated documentation. It
    /// refuses non-loopback and HTTPS URLs, and no production resolver calls
    /// it. Real local signer deployments use authenticated HTTPS.
    #[doc(hidden)]
    pub fn new_insecure_loopback_for_test(
        base_url: &str,
        admin_token: Option<String>,
        timeout: Duration,
    ) -> Result<Self> {
        Self::new_insecure_loopback_with_origin_for_test(base_url, admin_token, None, timeout)
    }

    #[doc(hidden)]
    pub fn new_insecure_loopback_with_origin_for_test(
        base_url: &str,
        admin_token: Option<String>,
        consumer_origin: Option<String>,
        timeout: Duration,
    ) -> Result<Self> {
        let mut config = Self::new_with_origin(base_url, admin_token, consumer_origin, timeout)?;
        config.enable_insecure_loopback_test_transport()?;
        Ok(config)
    }

    /// Build a native-contract client with an Ed25519 workload identity.
    pub fn new_with_workload(
        service_url: &str,
        caller_id: impl Into<String>,
        seed: [u8; 32],
        subject: Option<String>,
        timeout: Duration,
    ) -> Result<Self> {
        let (origin, cleartext, loopback) = validated_service_url(service_url)?;
        let (caller_id, subject) = validated_workload_identity(caller_id.into(), subject)?;
        let mut seed = seed;
        let workload = WorkloadCredential::new(caller_id, subject, seed);
        seed.zeroize();
        Ok(Self {
            origin,
            mount_path: String::new(),
            admin: Some(AdminCredential::Workload(Box::new(workload))),
            consumer_origin: None,
            timeout,
            cleartext,
            loopback,
            allow_insecure_loopback_http_for_test: false,
        })
    }

    #[doc(hidden)]
    pub fn new_insecure_loopback_with_workload_for_test(
        service_url: &str,
        caller_id: impl Into<String>,
        seed: [u8; 32],
        subject: Option<String>,
        timeout: Duration,
    ) -> Result<Self> {
        let mut config = Self::new_with_workload(service_url, caller_id, seed, subject, timeout)?;
        config.enable_insecure_loopback_test_transport()?;
        Ok(config)
    }

    /// Borrow the configured operator identity for one synchronous request.
    pub fn admin(&self) -> Option<CallerToken<'_>> {
        Some(match self.admin.as_ref()? {
            AdminCredential::Bearer(token) => self.consumer_origin.as_deref().map_or_else(
                || CallerToken::bearer(token.as_str()),
                |origin| CallerToken::browser(token.as_str(), Some(origin), None),
            ),
            AdminCredential::Workload(workload) => CallerToken::workload(workload),
        })
    }

    pub fn consumer_origin(&self) -> Option<&str> {
        self.consumer_origin.as_deref()
    }

    pub fn base_url(&self) -> String {
        format!("{}{}", self.origin, self.mount_path)
    }

    /// Canonical, path-free custody origin committed to by workload v2.
    pub(crate) fn workload_audience(&self) -> &str {
        &self.origin
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn uses_cleartext(&self) -> bool {
        self.cleartext
    }

    pub fn is_loopback(&self) -> bool {
        self.loopback
    }

    pub(crate) fn allows_insecure_loopback_http_for_test(&self) -> bool {
        self.allow_insecure_loopback_http_for_test
    }

    pub(crate) fn url(&self, path: &str, query: Option<&str>) -> String {
        format!(
            "{}{}{}",
            self.origin,
            self.mount_path,
            self.route(path, query)
        )
    }

    /// Exact path-and-query the native signer authenticates for workload proof.
    pub(crate) fn route(&self, path: &str, query: Option<&str>) -> String {
        let mut route = String::with_capacity(API_PREFIX.len() + path.len() + 32);
        route.push_str(API_PREFIX);
        route.push_str(path);
        if let Some(query) = query {
            route.push('?');
            route.push_str(query);
        }
        route
    }

    fn production(
        origin: String,
        admin: AdminCredential,
        consumer_origin: Option<String>,
        timeout: Duration,
        cleartext: bool,
        loopback: bool,
    ) -> Self {
        Self {
            origin,
            mount_path: String::new(),
            admin: Some(admin),
            consumer_origin,
            timeout,
            cleartext,
            loopback,
            allow_insecure_loopback_http_for_test: false,
        }
    }

    pub(crate) fn enable_insecure_loopback_test_transport(&mut self) -> Result<()> {
        if !self.cleartext || !self.loopback {
            bail!("the insecure test transport requires an HTTP loopback signer URL");
        }
        self.allow_insecure_loopback_http_for_test = true;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/signer_client/config/tests.rs"]
mod tests;
