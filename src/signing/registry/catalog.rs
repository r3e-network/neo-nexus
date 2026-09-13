//! Signer profile catalog and dynamic request dispatcher.

use std::{collections::BTreeMap, env, path::Path};

use anyhow::{bail, Context, Result};

use crate::signer_client::{
    Eip191FulfillmentRequest, Eip191FulfillmentSignature, KeyPublic, Outcome, RawSignRequest,
    RawSignature, SignRequest, Signature, SignerClient,
};

use super::{
    super::{SignerBackendProfile, SignerKeyRef},
    backend::ConfiguredSignerBackend,
    document::{normalized_id, read_registry_document, REGISTRY_VERSION},
    PROFILES_FILE_ENV,
};

/// A signer profile catalog plus the two control-plane routing roles.
///
/// Application signing deliberately has no process-wide default. A managed
/// node persists one [`SignerKeyRef`] and every signing operation resolves that
/// exact backend and key.
#[derive(Clone, Debug)]
pub struct SignerRegistry {
    backends: BTreeMap<String, ConfiguredSignerBackend>,
    console_backend: Option<String>,
    relay_backend: Option<String>,
}

impl SignerRegistry {
    pub fn empty() -> Self {
        Self {
            backends: BTreeMap::new(),
            console_backend: None,
            relay_backend: None,
        }
    }

    pub fn new(
        backends: impl IntoIterator<Item = ConfiguredSignerBackend>,
        console_backend: Option<String>,
        relay_backend: Option<String>,
    ) -> Result<Self> {
        let mut indexed = BTreeMap::new();
        for backend in backends {
            let id = backend.profile().id.clone();
            if indexed.insert(id.clone(), backend).is_some() {
                bail!("duplicate signer backend id {id}");
            }
        }
        let registry = Self {
            backends: indexed,
            console_backend: normalized_id(console_backend),
            relay_backend: normalized_id(relay_backend),
        };
        registry.validate_routes()?;
        Ok(registry)
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let document = read_registry_document(path)?;
        if document.version != REGISTRY_VERSION {
            bail!(
                "signer registry {} has version {}; expected {REGISTRY_VERSION}",
                path.display(),
                document.version
            );
        }
        if document.backends.is_empty() {
            bail!("signer registry {} has no backends", path.display());
        }
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let mut backends = Vec::with_capacity(document.backends.len());
        for backend in document.backends {
            backends.push(backend.open(base)?);
        }
        Self::new(backends, document.console_backend, document.relay_backend)
            .with_context(|| format!("invalid signer routes in {}", path.display()))
    }

    pub fn from_env_file() -> Result<Option<Self>> {
        match env::var(PROFILES_FILE_ENV) {
            Ok(value) if value.trim().is_empty() => bail!("{PROFILES_FILE_ENV} must not be blank"),
            Ok(value) => Self::from_file(Path::new(value.trim())).map(Some),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(env::VarError::NotUnicode(_)) => {
                bail!("{PROFILES_FILE_ENV} must contain valid Unicode")
            }
        }
    }

    pub fn profiles(&self) -> impl Iterator<Item = &SignerBackendProfile> {
        self.backends.values().map(ConfiguredSignerBackend::profile)
    }

    pub fn backend(&self, id: &str) -> Result<&ConfiguredSignerBackend> {
        self.backends
            .get(id.trim())
            .with_context(|| format!("signer backend {id:?} is not configured"))
    }

    /// Backend id → the one key it owns, for every backend whose key is known
    /// without asking a remote custody service.
    ///
    /// Lets a form fill in a key the workspace already knows instead of asking
    /// an operator to retype it and then rejecting the typo.
    pub fn sole_key_ids(&self) -> std::collections::BTreeMap<String, String> {
        self.backends
            .iter()
            .filter_map(|(id, backend)| Some((id.clone(), backend.sole_key_id()?)))
            .collect()
    }

    pub fn console_backend(&self) -> Result<&ConfiguredSignerBackend> {
        self.routed_backend(self.console_backend.as_deref(), "console")
    }

    pub fn relay_backend(&self) -> Result<&ConfiguredSignerBackend> {
        self.routed_backend(self.relay_backend.as_deref(), "public relay")
    }

    pub fn console_backend_id(&self) -> Option<&str> {
        self.console_backend.as_deref()
    }

    pub fn relay_backend_id(&self) -> Option<&str> {
        self.relay_backend.as_deref()
    }

    /// Dispatch an internal transaction request to the backend named by the
    /// key reference. Service profiles use their separate restricted signing
    /// identity; local wallets remain process-local.
    pub fn sign_transaction(
        &self,
        key: &SignerKeyRef,
        request: &SignRequest,
    ) -> Result<Outcome<Signature>> {
        ensure_request_key(key, &request.key_id)?;
        let outcome = match self.backend(&key.backend_id)? {
            ConfiguredSignerBackend::LocalWallet { signer, .. } => {
                signer.sign_transaction(request).map(Outcome::Allowed)
            }
            ConfiguredSignerBackend::LocalSigner { .. } => {
                bail!("the local signer is a native consensus-only gRPC backend; it cannot sign an application transaction")
            }
            backend @ ConfiguredSignerBackend::NeoOsService { .. } => {
                with_service_signer(backend, |client, credentials| {
                    client.sign_transaction_request(credentials, request)
                })
            }
        }?;
        ensure_outcome_key(key, &outcome, |signature| &signature.key_id)?;
        Ok(outcome)
    }

    pub fn sign_consensus(
        &self,
        key: &SignerKeyRef,
        request: &SignRequest,
    ) -> Result<Outcome<Signature>> {
        ensure_request_key(key, &request.key_id)?;
        let outcome = match self.backend(&key.backend_id)? {
            ConfiguredSignerBackend::LocalWallet { signer, .. } => {
                signer.sign_consensus(request).map(Outcome::Allowed)
            }
            ConfiguredSignerBackend::LocalSigner { .. } => {
                bail!("the local signer accepts structured SecureSign protobuf messages from the native node, not NeoOS HTTP SignRequest bytes")
            }
            backend @ ConfiguredSignerBackend::NeoOsService { .. } => {
                with_service_signer(backend, |client, credentials| {
                    client.sign_consensus_request(credentials, request)
                })
            }
        }?;
        ensure_outcome_key(key, &outcome, |signature| &signature.key_id)?;
        Ok(outcome)
    }

    pub fn sign_raw(
        &self,
        key: &SignerKeyRef,
        request: &RawSignRequest,
    ) -> Result<Outcome<RawSignature>> {
        ensure_request_key(key, &request.key_id)?;
        let outcome = match self.backend(&key.backend_id)? {
            ConfiguredSignerBackend::LocalWallet { signer, .. } => {
                signer.sign_raw(request).map(Outcome::Allowed)
            }
            ConfiguredSignerBackend::LocalSigner { .. } => {
                bail!("the local consensus signer has no raw-signing operation")
            }
            backend @ ConfiguredSignerBackend::NeoOsService { .. } => {
                with_service_signer(backend, |client, credentials| {
                    client.sign_raw_request(credentials, request)
                })
            }
        }?;
        ensure_outcome_key(key, &outcome, |signature| &signature.key_id)?;
        Ok(outcome)
    }

    /// Dispatch the structured NeoX oracle-fulfillment lane. A local NEP-6
    /// wallet cannot satisfy this secp256k1/EIP-191 contract and fails closed.
    pub fn sign_eip191_fulfillment(
        &self,
        key: &SignerKeyRef,
        request: &Eip191FulfillmentRequest,
    ) -> Result<Outcome<Eip191FulfillmentSignature>> {
        ensure_request_key(key, &request.key_id)?;
        let outcome = match self.backend(&key.backend_id)? {
            ConfiguredSignerBackend::LocalWallet { .. }
            | ConfiguredSignerBackend::LocalSigner { .. } => {
                bail!("the selected Neo N3 local backend does not support NeoX EIP-191 fulfillment signing")
            }
            backend @ ConfiguredSignerBackend::NeoOsService { .. } => {
                with_service_signer(backend, |client, credentials| {
                    client.sign_eip191_fulfillment(credentials, request)
                })
            }
        }?;
        ensure_outcome_key(key, &outcome, |signature| &signature.key_id)?;
        Ok(outcome)
    }

    pub fn key_info(&self, key: &SignerKeyRef) -> Result<Outcome<KeyPublic>> {
        let outcome = match self.backend(&key.backend_id)? {
            ConfiguredSignerBackend::LocalWallet { signer, .. } => {
                let identity = signer.key_info();
                ensure_request_key(key, &identity.key_id)?;
                Ok(Outcome::Allowed(identity))
            }
            ConfiguredSignerBackend::LocalSigner { config, .. } => {
                if key.key_id != config.public_key() {
                    bail!(
                        "local signer profile owns public key {}, not {}",
                        config.public_key(),
                        key.key_id
                    );
                }
                bail!("local signer account status is queried by the native SignClient gRPC plugin")
            }
            backend @ ConfiguredSignerBackend::NeoOsService { .. } => {
                with_service_signer(backend, |client, credentials| {
                    client.key_info(credentials, &key.key_id)
                })
            }
        }?;
        ensure_outcome_key(key, &outcome, |identity| &identity.key_id)?;
        Ok(outcome)
    }

    fn routed_backend(
        &self,
        id: Option<&str>,
        route_name: &str,
    ) -> Result<&ConfiguredSignerBackend> {
        let id = id.with_context(|| format!("no signer backend is selected for {route_name}"))?;
        self.backend(id)
    }

    fn validate_routes(&self) -> Result<()> {
        if self.backends.is_empty()
            && (self.console_backend.is_some() || self.relay_backend.is_some())
        {
            bail!("an empty signer registry cannot define routes");
        }
        if let Some(id) = &self.console_backend {
            if self.backend(id)?.service().is_none() {
                bail!("console backend {id} is not a NeoOS HTTP custody service");
            }
        }
        if let Some(id) = &self.relay_backend {
            let backend = self.backend(id)?;
            if backend.service().is_none() {
                bail!("public relay backend {id} is not a signer service");
            }
        }
        Ok(())
    }
}

fn with_service_signer<T>(
    backend: &ConfiguredSignerBackend,
    call: impl FnOnce(&SignerClient, &crate::signer_client::CallerToken<'_>) -> Result<Outcome<T>>,
) -> Result<Outcome<T>> {
    let service = backend
        .service()
        .context("the selected backend is not service-backed")?;
    let client = service.signing_client().with_context(|| {
        format!(
            "signer backend {} has no restricted signing identity",
            backend.profile().id
        )
    })?;
    let credentials = client
        .config()
        .admin()
        .context("the restricted signing identity disappeared")?;
    call(client, &credentials)
}

fn ensure_request_key(reference: &SignerKeyRef, request_key_id: &str) -> Result<()> {
    if reference.key_id != request_key_id.trim() {
        bail!(
            "signer key reference names {}, but the request names {}",
            reference.key_id,
            request_key_id.trim()
        );
    }
    Ok(())
}

pub(super) fn ensure_outcome_key<T>(
    reference: &SignerKeyRef,
    outcome: &Outcome<T>,
    key_id: impl FnOnce(&T) -> &str,
) -> Result<()> {
    if let Outcome::Allowed(payload) = outcome {
        let returned = key_id(payload).trim();
        if returned != reference.key_id {
            bail!(
                "signer backend {} returned key {returned:?} for requested key {}",
                reference.backend_id,
                reference.key_id
            );
        }
    }
    Ok(())
}
