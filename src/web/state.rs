//! Server state shared by every web handler: the workspace repository, the
//! data directory (workspace children such as managed configs and logs live
//! beside the database), the authentication store, the one process supervisor
//! the whole server shares, and the explicitly routed signer registry.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{anyhow, bail, Context, Result};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::{
    repository::Repository,
    signer_client::{
        CallerToken, SignerClient, SignerConfig, CALLER_ID_ENV, TOKEN_FILE_ENV, URL_ENV,
        WORKLOAD_KEY_FILE_ENV,
    },
    signing::{
        ConfiguredSignerBackend, LocalWalletSigner, ServiceSignerBackend, SignerBackendKind,
        SignerBackendProfile, SignerCapabilities, SignerRegistry,
    },
    supervisor::ProcessSupervisor,
};

#[cfg(test)]
use crate::signing::{LocalSignerConfig, LocalWalletConfig};

use super::auth::{AuthStore, WebSecurity};
use super::jobs::Jobs;
use crate::core::workspace_commands::WorkspaceCommands;
use crate::core::workspace_queries::WorkspaceQueries;

const SIGNER_RELAY_CONCURRENCY: usize = 32;

#[derive(Clone)]
pub struct WebState {
    repository: Repository,
    pub workspace: WorkspaceQueries,
    pub commands: WorkspaceCommands,
    pub data_dir: PathBuf,
    pub auth: AuthStore,
    web_security: WebSecurity,
    processes: Arc<Mutex<ProcessSupervisor>>,
    /// Long work that outlives the request which started it.
    pub jobs: Jobs,
    /// Concurrent named signing backends with explicit console, relay, and
    /// internal-signing routes. There is never an automatic fallback.
    custody: Custody,
    /// Public caller traffic may wait on custody, but it may never grow an
    /// unbounded blocking-work queue in this process.
    signer_relay: Arc<Semaphore>,
}

impl WebState {
    /// Build server state and resolve signer configuration once.
    ///
    /// A malformed, partial, or mixed-backend signer configuration is an error
    /// here, before a listener or background engine starts. Runtime requests may
    /// see a backend fail, but they never select a fallback implicitly.
    pub fn new(
        repository: Repository,
        data_dir: PathBuf,
        auth: AuthStore,
        web_security: WebSecurity,
    ) -> Result<Self> {
        Self::with_supervisor(
            repository,
            data_dir,
            auth,
            web_security,
            Arc::new(Mutex::new(ProcessSupervisor::default())),
        )
    }

    /// Wrap a supervisor the caller already holds. The server uses this so the
    /// browser and the supervision engine act on the same handles — two
    /// supervisors would mean a node the loop started could not be stopped from
    /// the page, and the other way round too.
    pub fn with_supervisor(
        repository: Repository,
        data_dir: PathBuf,
        auth: AuthStore,
        web_security: WebSecurity,
        processes: Arc<Mutex<ProcessSupervisor>>,
    ) -> Result<Self> {
        let workspace = WorkspaceQueries::new(repository.clone());
        let commands = WorkspaceCommands::new(repository.clone());
        Ok(Self {
            repository,
            workspace,
            commands,
            data_dir,
            auth,
            web_security,
            processes,
            jobs: Jobs::default(),
            custody: Custody::from_env()?,
            signer_relay: Arc::new(Semaphore::new(SIGNER_RELAY_CONCURRENCY)),
        })
    }

    /// Install a signing backend the caller already resolved.
    #[must_use]
    pub fn with_custody(mut self, custody: Custody) -> Self {
        self.custody = custody;
        self
    }

    /// Override the public relay bound. This is primarily a deterministic test
    /// seam; production uses [`SIGNER_RELAY_CONCURRENCY`].
    #[doc(hidden)]
    #[must_use]
    pub fn with_signer_relay_limit(mut self, limit: usize) -> Self {
        self.signer_relay = Arc::new(Semaphore::new(limit.max(1)));
        self
    }

    pub fn web_security(&self) -> &WebSecurity {
        &self.web_security
    }

    pub fn session_cookie(&self, session_id: &str) -> String {
        self.auth
            .session_cookie(session_id, self.web_security.secure_cookies())
    }

    pub fn clear_session_cookie(&self) -> String {
        self.auth.clear_cookie(self.web_security.secure_cookies())
    }

    pub(crate) fn try_signer_relay_permit(&self) -> Option<OwnedSemaphorePermit> {
        Arc::clone(&self.signer_relay).try_acquire_owned().ok()
    }

    /// A subdirectory beside the database, mirroring the GUI and CLI
    /// conventions: managed configs under `nodes/`, supervised logs under
    /// `logs/`.
    pub fn workspace_child_dir(&self, child: &str) -> PathBuf {
        self.data_dir.join(child)
    }

    /// The shared supervisor handle, for handing to the supervision engine.
    ///
    /// One supervisor for the life of the server, shared as an `Arc` so every
    /// handler and the background loop reach the same handles. A per-request
    /// supervisor could not stop what another request started, and its `Drop`
    /// would terminate the node the request had just launched.
    pub fn shared_supervisor(&self) -> Arc<Mutex<ProcessSupervisor>> {
        Arc::clone(&self.processes)
    }

    /// The same workspace as the supervision engine sees it. Handlers go through
    /// this so a browser start and a watchdog restart run the identical
    /// pipeline against the identical supervisor, rather than each keeping its
    /// own copy of the steps.
    pub fn engine_state(&self) -> crate::supervision::EngineState {
        crate::supervision::EngineState {
            repository: self.repository.clone(),
            data_dir: self.data_dir.clone(),
            supervisor: self.shared_supervisor(),
            signer_registry: self.custody.registry().clone(),
        }
    }

    /// The shared supervisor, locked for one short operation.
    ///
    /// A poisoned lock is recovered rather than propagated: the map it guards
    /// holds process handles, and refusing every future start because one
    /// handler panicked somewhere else would turn a single fault into an
    /// unmanageable fleet.
    pub fn supervisor(&self) -> MutexGuard<'_, ProcessSupervisor> {
        self.processes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Whether the workbench itself is supervising this node — i.e. whether a
    /// stop can reach the process rather than only the row.
    pub fn is_supervised(&self, node_id: &str) -> bool {
        self.supervisor().is_managing(node_id)
    }

    /// The selected service-backed signer client, ready to be asked.
    ///
    /// A borrowed client: a handler that crosses into
    /// [`tokio::task::spawn_blocking`] clones it, which is an `Arc` copy of a
    /// configuration and a connection agent rather than a second path anywhere.
    /// A local-wallet backend returns an explicit error because it is never
    /// exposed through the public relay or remote administration handlers.
    pub fn signer(&self) -> Result<&SignerClient> {
        self.custody.relay_client()
    }

    /// Custody as the page sees it, including the deliberately unconfigured
    /// state.
    pub fn custody(&self) -> &Custody {
        &self.custody
    }
}

/// Concurrent, named signing backends used by the workbench.
///
/// A profile registry can contain all three backend families at once. The
/// console and public relay each name one backend explicitly, while internal
/// signing always uses a backend-qualified key reference. A failure never
/// changes any of those routes.
#[derive(Clone)]
pub struct Custody {
    registry: SignerRegistry,
}

impl Custody {
    fn from_env() -> Result<Self> {
        Ok(Self {
            registry: SignerRegistry::from_process_environment()?,
        })
    }

    #[cfg(test)]
    fn resolve(
        selected: Option<SignerBackendKind>,
        local_wallet: Option<LocalWalletConfig>,
        local_signer: Option<LocalSignerConfig>,
        service: Option<SignerConfig>,
    ) -> Result<Self> {
        Ok(Self {
            registry: SignerRegistry::from_legacy_components(
                selected,
                local_wallet,
                local_signer,
                service,
            )?,
        })
    }

    pub fn from_registry(registry: SignerRegistry) -> Self {
        Self { registry }
    }

    pub fn unconfigured() -> Self {
        Self {
            registry: SignerRegistry::empty(),
        }
    }

    /// Custody pointed at a service the caller resolved — the seam a test uses to
    /// talk to a stub, and the only way to install a client without an
    /// environment.
    pub fn serving(client: SignerClient) -> Result<Self> {
        if client.config().is_loopback() || client.config().uses_cleartext() {
            return Self::single_compatible_neo_os_service(client);
        }
        Self::single_neo_os_service(client.config().clone())
    }

    pub fn local_wallet(signer: LocalWalletSigner) -> Result<Self> {
        Self::single_local_wallet(signer)
    }

    pub fn kind(&self) -> Option<SignerBackendKind> {
        if let Ok(backend) = self.registry.console_backend() {
            return Some(backend.profile().kind);
        }
        let mut profiles = self.registry.profiles();
        let only = profiles.next()?;
        profiles.next().is_none().then_some(only.kind)
    }

    pub fn capabilities(&self) -> SignerCapabilities {
        self.registry
            .console_backend()
            .map(ConfiguredSignerBackend::capabilities)
            .unwrap_or_else(|_| SignerCapabilities::local_wallet(false, false, false))
    }

    pub fn local_wallet_signer(&self) -> Option<&LocalWalletSigner> {
        self.registry
            .console_backend()
            .ok()
            .and_then(ConfiguredSignerBackend::local_wallet_signer)
    }

    pub fn profiles(&self) -> impl Iterator<Item = &SignerBackendProfile> {
        self.registry.profiles()
    }

    pub fn registry(&self) -> &SignerRegistry {
        &self.registry
    }

    /// The client, or why there is none, in words an operator can act on.
    ///
    /// Both refusals are `Err` rather than an `Option`: a handler that forgot to
    /// ask would then answer with an empty list, which reads as "no keys" and is a
    /// lie about a custody service that may be running fine three hosts away.
    pub fn client(&self) -> Result<&SignerClient> {
        self.registry
            .console_backend()
            .and_then(|backend| {
                backend.service().context(
                    "the console backend is a local wallet and has no remote administration surface",
                )
            })
            .map(ServiceSignerBackend::admin_client)
            .map_err(|error| anyhow!("{error}"))
    }

    pub fn relay_client(&self) -> Result<&SignerClient> {
        self.registry
            .relay_backend()
            .and_then(|backend| {
                backend
                    .service()
                    .context("the public relay route is not service-backed")
            })
            .map(ServiceSignerBackend::admin_client)
            .map_err(|error| anyhow!("{error}"))
    }

    /// The console's own way to ask: a client and the credential to ask with.
    ///
    /// Owned rather than borrowed because the management routes run on a blocking
    /// thread, and a credential that had to stay on the request thread would put
    /// the vault call back where it does not belong.
    ///
    /// [`SignerConfig::from_env`] guarantees production state cannot have a URL
    /// without exactly one admin identity. The `None` branch remains for tests
    /// and direct library constructors, which may build a relay-only client.
    pub fn admin(&self) -> Result<Admin> {
        let client = self.client()?.clone();
        if client.config().admin().is_none() {
            bail!(
                "{URL_ENV} is configured without an admin identity. Set exactly one of \
                 {TOKEN_FILE_ENV}, or {CALLER_ID_ENV} with {WORKLOAD_KEY_FILE_ENV}, to an \
                 identity holding `admin` and granted every key (§5.1)"
            );
        }
        Ok(Admin { client })
    }

    /// Resolve the console's credential and run one call with it, off the request
    /// thread.
    ///
    /// The client is blocking on purpose — it follows the federation and alert
    /// transports, and the wait is for *another process*, which can take the whole
    /// timeout. That is only safe because nothing in `src/web` calls it directly:
    /// every custody page and control runs through here, so the one place that
    /// decides "hand this to a worker thread" is one place.
    ///
    /// A policy denial arrives as `Err` too. §5 says a refusal is a completed
    /// conversation, and the relay in `signer_api` keeps it that way; the console
    /// cannot, since what it shows an operator is a sentence either way — but the
    /// text is the service's own code and message, never "the request failed".
    pub async fn ask<T, Call>(&self, call: Call) -> Result<T>
    where
        Call: FnOnce(&Admin) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let admin = self.admin()?;
        tokio::task::spawn_blocking(move || call(&admin))
            .await
            .map_err(|joined| anyhow!("the custody request thread did not finish: {joined}"))?
    }

    fn single_local_wallet(signer: LocalWalletSigner) -> Result<Self> {
        let id = SignerBackendKind::LocalWallet.slug().to_string();
        let profile = SignerBackendProfile::new(
            &id,
            SignerBackendKind::LocalWallet.label(),
            SignerBackendKind::LocalWallet,
        )?;
        let backend = ConfiguredSignerBackend::local_wallet(profile, signer)?;
        Ok(Self {
            registry: SignerRegistry::new([backend], Some(id), None)?,
        })
    }

    fn single_neo_os_service(config: SignerConfig) -> Result<Self> {
        let kind = SignerBackendKind::NeoOsService;
        let id = kind.slug().to_string();
        let profile = SignerBackendProfile::new(&id, kind.label(), kind)?;
        let service = ServiceSignerBackend::new(SignerClient::new(config), None);
        let backend = ConfiguredSignerBackend::neo_os_service(profile, service)?;
        Ok(Self {
            registry: SignerRegistry::new([backend], Some(id.clone()), Some(id))?,
        })
    }

    fn single_compatible_neo_os_service(client: SignerClient) -> Result<Self> {
        let kind = SignerBackendKind::NeoOsService;
        let id = kind.slug().to_string();
        let profile = SignerBackendProfile::new(&id, kind.label(), kind)?;
        let backend = ConfiguredSignerBackend::neo_os_service_compatible(
            profile,
            ServiceSignerBackend::new(client, None),
        )?;
        Ok(Self {
            registry: SignerRegistry::new([backend], Some(id.clone()), Some(id))?,
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/web/state/tests.rs"]
mod tests;

/// A custody service and the admin credential this process presents to it.
///
/// The console's half of the split §5.1 was written to make possible: this is the
/// credential that can move a boundary, and it is deliberately not the one that
/// can sign — an admin token used on a sign route is refused by name, so a
/// compromised console credential cannot become a signing oracle.
pub struct Admin {
    client: SignerClient,
}

impl Admin {
    /// The credential, borrowed for one call.
    pub fn credentials(&self) -> Result<CallerToken<'_>> {
        self.client.config().admin().with_context(|| {
            format!(
                "the signer admin identity disappeared; configure {TOKEN_FILE_ENV}, or \
                 {CALLER_ID_ENV} with {WORKLOAD_KEY_FILE_ENV}"
            )
        })
    }

    /// The handle a management call is made through.
    pub fn client(&self) -> &SignerClient {
        &self.client
    }

    /// The endpoint the console is talking to, for the page that says so.
    pub fn base_url(&self) -> String {
        self.client.config().base_url()
    }

    /// Whether this direct, caller-supplied client uses cleartext. Production
    /// registry and environment resolution reject cleartext signer profiles;
    /// this remains observable for the in-process compatibility test seam.
    pub fn uses_cleartext(&self) -> bool {
        self.client.config().uses_cleartext()
    }
}
