//! Named signer profiles and explicit, backend-qualified dispatch.
//!
//! A registry may hold node-local wallets, native `SecureSign` gRPC deployments,
//! and NeoOS HTTP custody services at the same time.  Those protocols are never
//! inferred from endpoint location or routed through one another.

use std::{
    collections::BTreeMap,
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::signer_client::{
    Eip191FulfillmentRequest, Eip191FulfillmentSignature, KeyPublic, Outcome, RawSignRequest,
    RawSignature, SignRequest, Signature, SignerClient, SignerConfig, SignerProfileInput,
};

use super::{
    LocalSignerConfig, LocalWalletConfig, LocalWalletSigner, SignerBackendKind,
    SignerBackendProfile, SignerCapabilities, SignerKeyRef,
};

pub const PROFILES_FILE_ENV: &str = "NEONEXUS_SIGNER_PROFILES_FILE";
const REGISTRY_VERSION: u32 = 1;
const MAX_REGISTRY_BYTES: u64 = 256 * 1024;

/// A service-backed profile keeps its admin and signing identities separate.
/// The first is used only by the operator console; the second is optional and
/// must be a least-privilege `sign` caller for internal dispatch.
#[derive(Clone)]
pub struct ServiceSignerBackend {
    admin: SignerClient,
    signing: Option<SignerClient>,
}

impl ServiceSignerBackend {
    pub fn new(admin: SignerClient, signing: Option<SignerClient>) -> Self {
        Self { admin, signing }
    }

    pub fn admin_client(&self) -> &SignerClient {
        &self.admin
    }

    pub fn signing_client(&self) -> Option<&SignerClient> {
        self.signing.as_ref()
    }
}

impl std::fmt::Debug for ServiceSignerBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServiceSignerBackend")
            .field("admin", &self.admin)
            .field("signing_identity_configured", &self.signing.is_some())
            .finish()
    }
}

/// One configured backend and its stable, non-secret identity.
#[derive(Clone)]
pub enum ConfiguredSignerBackend {
    LocalWallet {
        profile: SignerBackendProfile,
        signer: Box<LocalWalletSigner>,
    },
    LocalSigner {
        profile: SignerBackendProfile,
        config: LocalSignerConfig,
    },
    NeoOsService {
        profile: SignerBackendProfile,
        service: ServiceSignerBackend,
        /// Optional node-facing SecureSign-compatible bridge. The HTTP
        /// service remains the custody/control plane; native neo-cli consumes
        /// this separate gRPC endpoint.
        node_signer: Option<LocalSignerConfig>,
    },
}

impl std::fmt::Debug for ConfiguredSignerBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ConfiguredSignerBackend")
            .field("profile", self.profile())
            .field("capabilities", &self.capabilities())
            .finish()
    }
}

impl ConfiguredSignerBackend {
    pub fn local_wallet(profile: SignerBackendProfile, signer: LocalWalletSigner) -> Result<Self> {
        require_kind(&profile, SignerBackendKind::LocalWallet)?;
        Ok(Self::LocalWallet {
            profile,
            signer: Box::new(signer),
        })
    }

    pub fn local_signer(profile: SignerBackendProfile, config: LocalSignerConfig) -> Result<Self> {
        require_kind(&profile, SignerBackendKind::LocalSigner)?;
        Ok(Self::LocalSigner { profile, config })
    }

    pub fn neo_os_service(
        profile: SignerBackendProfile,
        service: ServiceSignerBackend,
    ) -> Result<Self> {
        Self::neo_os_service_with_node_signer(profile, service, None)
    }

    pub fn neo_os_service_with_node_signer(
        profile: SignerBackendProfile,
        service: ServiceSignerBackend,
        node_signer: Option<LocalSignerConfig>,
    ) -> Result<Self> {
        require_kind(&profile, SignerBackendKind::NeoOsService)?;
        validate_service_transports(&profile, &service, Some(false), true)?;
        Ok(Self::NeoOsService {
            profile,
            service,
            node_signer,
        })
    }

    /// Compatibility seam for an in-process NeoOS HTTP test stub. Production
    /// registry documents and environment resolution always require the remote
    /// authenticated HTTPS topology.
    pub(crate) fn neo_os_service_compatible(
        profile: SignerBackendProfile,
        service: ServiceSignerBackend,
    ) -> Result<Self> {
        require_kind(&profile, SignerBackendKind::NeoOsService)?;
        validate_service_transports(&profile, &service, None, false)?;
        Ok(Self::NeoOsService {
            profile,
            service,
            node_signer: None,
        })
    }

    pub fn profile(&self) -> &SignerBackendProfile {
        match self {
            Self::LocalWallet { profile, .. }
            | Self::LocalSigner { profile, .. }
            | Self::NeoOsService { profile, .. } => profile,
        }
    }

    pub fn capabilities(&self) -> SignerCapabilities {
        match self {
            Self::LocalWallet { signer, .. } => signer.capabilities(),
            Self::LocalSigner { .. } => SignerCapabilities::local_signer(),
            Self::NeoOsService { .. } => SignerCapabilities::service(),
        }
    }

    pub fn local_wallet_signer(&self) -> Option<&LocalWalletSigner> {
        match self {
            Self::LocalWallet { signer, .. } => Some(signer.as_ref()),
            Self::LocalSigner { .. } | Self::NeoOsService { .. } => None,
        }
    }

    pub fn service(&self) -> Option<&ServiceSignerBackend> {
        match self {
            Self::NeoOsService { service, .. } => Some(service),
            Self::LocalWallet { .. } | Self::LocalSigner { .. } => None,
        }
    }

    pub fn local_signer_config(&self) -> Option<&LocalSignerConfig> {
        match self {
            Self::LocalSigner { config, .. } => Some(config),
            Self::LocalWallet { .. } | Self::NeoOsService { .. } => None,
        }
    }

    /// The gRPC bridge a native node may use for this NeoOS custody profile.
    /// Absence is explicit: application HTTP signing may still work, but a
    /// consensus node launch must fail instead of inventing a transport.
    pub fn neo_os_node_signer_config(&self) -> Option<&LocalSignerConfig> {
        match self {
            Self::NeoOsService { node_signer, .. } => node_signer.as_ref(),
            Self::LocalWallet { .. } | Self::LocalSigner { .. } => None,
        }
    }
}

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

fn ensure_outcome_key<T>(
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

fn require_kind(profile: &SignerBackendProfile, expected: SignerBackendKind) -> Result<()> {
    if profile.kind != expected {
        bail!(
            "signer profile {} declares {}, but its backend is {}",
            profile.id,
            profile.kind,
            expected
        );
    }
    Ok(())
}

fn validate_service_transports(
    profile: &SignerBackendProfile,
    service: &ServiceSignerBackend,
    expected_loopback: Option<bool>,
    require_https: bool,
) -> Result<()> {
    let admin = service.admin_client().config();
    for (role, client) in [
        ("admin", Some(service.admin_client())),
        ("signing", service.signing_client()),
    ] {
        let Some(client) = client else {
            continue;
        };
        let config = client.config();
        if require_https && config.uses_cleartext() {
            bail!(
                "{} profile {} requires HTTPS for its {role} transport",
                profile.kind,
                profile.id
            );
        }
        if expected_loopback.is_some_and(|expected| config.is_loopback() != expected) {
            let location = if expected_loopback == Some(true) {
                "a loopback"
            } else {
                "a non-loopback"
            };
            bail!(
                "{} profile {} requires {location} {role} origin",
                profile.kind,
                profile.id
            );
        }
        if config.base_url() != admin.base_url() {
            bail!(
                "signer profile {} must use one endpoint for its admin and signing identities",
                profile.id
            );
        }
    }
    Ok(())
}

fn normalized_id(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryDocument {
    version: u32,
    #[serde(default)]
    console_backend: Option<String>,
    #[serde(default)]
    relay_backend: Option<String>,
    backends: Vec<BackendDocument>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendDocument {
    id: String,
    label: String,
    kind: String,

    #[serde(default)]
    wallet_path: Option<String>,
    #[serde(default)]
    password_file: Option<String>,
    #[serde(default)]
    account: Option<String>,
    #[serde(default)]
    network: Option<String>,
    #[serde(default)]
    network_magic: Option<u32>,
    #[serde(default)]
    allow_transaction: Option<bool>,
    #[serde(default)]
    allow_consensus: Option<bool>,
    #[serde(default)]
    allow_raw: Option<bool>,

    #[serde(default)]
    endpoint: Option<String>,
    #[serde(default)]
    public_key: Option<String>,

    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    timeout_seconds: Option<u64>,
    #[serde(default)]
    admin_token_file: Option<String>,
    #[serde(default)]
    admin_caller_id: Option<String>,
    #[serde(default)]
    admin_workload_key_file: Option<String>,
    #[serde(default)]
    admin_workload_subject: Option<String>,
    #[serde(default)]
    admin_origin: Option<String>,
    #[serde(default)]
    signing_token_file: Option<String>,
    #[serde(default)]
    signing_caller_id: Option<String>,
    #[serde(default)]
    signing_workload_key_file: Option<String>,
    #[serde(default)]
    signing_workload_subject: Option<String>,
}

impl BackendDocument {
    fn open(self, base: &Path) -> Result<ConfiguredSignerBackend> {
        let profile_id = self.id.clone();
        let kind = self.kind.parse::<SignerBackendKind>()?;
        let profile = SignerBackendProfile::new(&self.id, &self.label, kind)?;
        match kind {
            SignerBackendKind::LocalWallet => self.open_wallet(profile, base),
            SignerBackendKind::LocalSigner => self.open_local_signer(profile),
            SignerBackendKind::NeoOsService => self.open_service(profile, base),
        }
        .with_context(|| format!("failed to open signer profile {profile_id}"))
    }

    fn open_wallet(
        self,
        profile: SignerBackendProfile,
        base: &Path,
    ) -> Result<ConfiguredSignerBackend> {
        if self.has_service_fields() || self.has_local_signer_fields() {
            bail!("local-wallet profile contains fields for another signer protocol");
        }
        let config = LocalWalletConfig {
            wallet_path: profile_path(base, &required(self.wallet_path, "wallet_path")?),
            password_file: profile_path(base, &required(self.password_file, "password_file")?),
            account: normalized_id(self.account),
            network: required(self.network, "network")?.trim().to_string(),
            network_magic: self
                .network_magic
                .context("local-wallet profile requires network_magic")?,
            allow_transaction: self.allow_transaction.unwrap_or(true),
            allow_consensus: self.allow_consensus.unwrap_or(false),
            allow_raw: self.allow_raw.unwrap_or(false),
        };
        if config.network_magic == 0 {
            bail!("local-wallet profile network_magic must be greater than zero");
        }
        if config.network.is_empty()
            || config.network.len() > 64
            || config.network.chars().any(char::is_control)
        {
            bail!("local-wallet profile network must be 1..=64 printable characters");
        }
        ConfiguredSignerBackend::local_wallet(profile, LocalWalletSigner::open(config)?)
    }

    fn open_local_signer(self, profile: SignerBackendProfile) -> Result<ConfiguredSignerBackend> {
        if self.has_wallet_fields() || self.has_service_fields() {
            bail!("local-signer profile contains local-wallet or NeoOS HTTP fields");
        }
        let endpoint = required(self.endpoint, "endpoint")?;
        let public_key = required(self.public_key, "public_key")?;
        let network_magic = self
            .network_magic
            .context("local-signer profile requires network_magic")?;
        ConfiguredSignerBackend::local_signer(
            profile,
            LocalSignerConfig::new(endpoint, public_key, network_magic)?,
        )
    }

    fn open_service(
        mut self,
        profile: SignerBackendProfile,
        base: &Path,
    ) -> Result<ConfiguredSignerBackend> {
        if self.has_wallet_fields() {
            bail!("neo-os-service profile contains local-wallet fields");
        }
        let node_signer = match (
            self.endpoint.take(),
            self.public_key.take(),
            self.network_magic.take(),
        ) {
            (None, None, None) => None,
            (Some(endpoint), Some(public_key), Some(network_magic)) => {
                Some(LocalSignerConfig::new(endpoint, public_key, network_magic)?)
            }
            _ => bail!(
                "neo-os-service node bridge requires endpoint, public_key, and network_magic together"
            ),
        };
        let url = required(self.url.clone(), "url")?.trim().to_string();
        let timeout = Duration::from_secs(self.timeout_seconds.unwrap_or(10));
        if timeout.is_zero() {
            bail!("service profile timeout_seconds must be greater than zero");
        }
        let has_signing_identity = self.signing_token_file.is_some()
            || self.signing_caller_id.is_some()
            || self.signing_workload_key_file.is_some()
            || self.signing_workload_subject.is_some();
        ensure_distinct_profile_files(
            base,
            self.admin_token_file
                .as_deref()
                .or(self.admin_workload_key_file.as_deref()),
            self.signing_token_file
                .as_deref()
                .or(self.signing_workload_key_file.as_deref()),
        )?;
        let admin_caller_id = normalized_id(self.admin_caller_id.clone());
        let signing_caller_id = normalized_id(self.signing_caller_id.clone());
        if admin_caller_id.is_some() && admin_caller_id == signing_caller_id {
            bail!("admin_caller_id and signing_caller_id must name different callers");
        }
        let admin = SignerClient::new(SignerConfig::from_profile(SignerProfileInput {
            service_url: url.clone(),
            token_file: self
                .admin_token_file
                .as_deref()
                .map(|path| profile_path(base, path)),
            caller_id: admin_caller_id,
            workload_key_file: self
                .admin_workload_key_file
                .as_deref()
                .map(|path| profile_path(base, path)),
            workload_subject: normalized_id(self.admin_workload_subject.clone()),
            consumer_origin: normalized_id(self.admin_origin.clone()),
            timeout,
        })?);

        let signing = has_signing_identity
            .then(|| {
                SignerConfig::from_profile(SignerProfileInput {
                    service_url: url,
                    token_file: self
                        .signing_token_file
                        .as_deref()
                        .map(|path| profile_path(base, path)),
                    caller_id: signing_caller_id,
                    workload_key_file: self
                        .signing_workload_key_file
                        .as_deref()
                        .map(|path| profile_path(base, path)),
                    workload_subject: normalized_id(self.signing_workload_subject),
                    consumer_origin: None,
                    timeout,
                })
                .map(SignerClient::new)
            })
            .transpose()?;
        let service = ServiceSignerBackend::new(admin, signing);
        ConfiguredSignerBackend::neo_os_service_with_node_signer(profile, service, node_signer)
    }

    fn has_wallet_fields(&self) -> bool {
        self.wallet_path.is_some()
            || self.password_file.is_some()
            || self.account.is_some()
            || self.network.is_some()
            || self.allow_transaction.is_some()
            || self.allow_consensus.is_some()
            || self.allow_raw.is_some()
    }

    fn has_local_signer_fields(&self) -> bool {
        self.endpoint.is_some() || self.public_key.is_some()
    }

    fn has_service_fields(&self) -> bool {
        self.url.is_some()
            || self.timeout_seconds.is_some()
            || self.admin_token_file.is_some()
            || self.admin_caller_id.is_some()
            || self.admin_workload_key_file.is_some()
            || self.admin_workload_subject.is_some()
            || self.admin_origin.is_some()
            || self.signing_token_file.is_some()
            || self.signing_caller_id.is_some()
            || self.signing_workload_key_file.is_some()
            || self.signing_workload_subject.is_some()
    }
}

fn read_registry_document(path: &Path) -> Result<RegistryDocument> {
    let file = File::open(path)
        .with_context(|| format!("failed to open signer registry {}", path.display()))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("failed to inspect signer registry {}", path.display()))?;
    if !metadata.is_file() {
        bail!("signer registry {} is not a regular file", path.display());
    }
    if metadata.len() > MAX_REGISTRY_BYTES {
        bail!(
            "signer registry {} exceeds the {MAX_REGISTRY_BYTES}-byte limit",
            path.display()
        );
    }
    let mut text = String::new();
    file.take(MAX_REGISTRY_BYTES + 1)
        .read_to_string(&mut text)
        .with_context(|| format!("failed to read signer registry {}", path.display()))?;
    if text.len() as u64 > MAX_REGISTRY_BYTES {
        bail!(
            "signer registry {} exceeds the {MAX_REGISTRY_BYTES}-byte limit",
            path.display()
        );
    }
    toml::from_str(&text)
        .with_context(|| format!("signer registry {} is not valid TOML", path.display()))
}

fn profile_path(base: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw.trim());
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn ensure_distinct_profile_files(
    base: &Path,
    admin: Option<&str>,
    signing: Option<&str>,
) -> Result<()> {
    let (Some(admin), Some(signing)) = (admin, signing) else {
        return Ok(());
    };
    let admin = std::fs::canonicalize(profile_path(base, admin))
        .context("failed to resolve the admin credential file")?;
    let signing = std::fs::canonicalize(profile_path(base, signing))
        .context("failed to resolve the signing credential file")?;
    #[cfg(windows)]
    let same = admin
        .to_string_lossy()
        .eq_ignore_ascii_case(&signing.to_string_lossy());
    #[cfg(not(windows))]
    let same = admin == signing;
    const MAX_CREDENTIAL_BYTES: u64 = 4 * 1024;
    let admin_secret =
        crate::secret_file::read_secret(&admin, MAX_CREDENTIAL_BYTES, "signer admin identity")?;
    let signing_secret =
        crate::secret_file::read_secret(&signing, MAX_CREDENTIAL_BYTES, "signer signing identity")?;
    if same || credential_line(&admin_secret) == credential_line(&signing_secret) {
        bail!("admin and signing identities must use different credential files and values");
    }
    Ok(())
}

fn credential_line(bytes: &[u8]) -> &[u8] {
    bytes
        .strip_suffix(b"\r\n")
        .or_else(|| bytes.strip_suffix(b"\n"))
        .unwrap_or(bytes)
}

fn required(value: Option<String>, field: &str) -> Result<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .with_context(|| format!("signer profile requires {field}"))
}

#[cfg(test)]
mod response_tests {
    use super::*;

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
