//! Server state shared by every web handler: the workspace repository, the
//! data directory (workspace children such as managed configs and logs live
//! beside the database), the authentication store, and the one process
//! supervisor the whole server shares.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};

use crate::{repository::Repository, signer_client::SignerClient, supervisor::ProcessSupervisor};

use super::auth::AuthStore;
use super::jobs::Jobs;

#[derive(Clone)]
pub struct WebState {
    pub repository: Repository,
    pub data_dir: PathBuf,
    pub auth: AuthStore,
    processes: Arc<Mutex<ProcessSupervisor>>,
    /// Long work that outlives the request which started it.
    pub jobs: Jobs,
    signer: SignerHandle,
}

impl WebState {
    pub fn new(repository: Repository, data_dir: PathBuf, auth: AuthStore) -> Self {
        Self::with_supervisor(
            repository,
            data_dir,
            auth,
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
        processes: Arc<Mutex<ProcessSupervisor>>,
    ) -> Self {
        Self {
            repository,
            data_dir,
            auth,
            processes,
            jobs: Jobs::default(),
            signer: SignerHandle::default(),
        }
    }

    /// Inject a configured signer client, primarily for composed deployments
    /// and end-to-end tests that do not source process environment.
    pub fn with_signer_client(mut self, client: SignerClient) -> Self {
        self.signer = SignerHandle::configured(client);
        self
    }

    /// Resolve the production signer profile lazily from process
    /// configuration. Tests and embedded users stay explicitly unconfigured
    /// unless they opt into this boundary.
    pub fn with_signer_from_env(mut self) -> Self {
        self.signer = SignerHandle::from_env();
        self
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

    /// Resolve the separately deployed custody client once.
    ///
    /// An absent or invalid signer configuration does not stop fleet
    /// operations. The signer page reports it explicitly and every signer
    /// control remains unavailable.
    pub fn signer_client(&self) -> Result<Option<SignerClient>, String> {
        self.signer.resolve()
    }
}

#[derive(Clone)]
struct SignerHandle {
    resolved: Arc<OnceLock<Result<Option<SignerClient>, String>>>,
}

impl SignerHandle {
    fn from_env() -> Self {
        Self {
            resolved: Arc::new(OnceLock::new()),
        }
    }

    fn configured(client: SignerClient) -> Self {
        let resolved = OnceLock::new();
        let _ = resolved.set(Ok(Some(client)));
        Self {
            resolved: Arc::new(resolved),
        }
    }

    fn resolve(&self) -> Result<Option<SignerClient>, String> {
        self.resolved
            .get_or_init(|| {
                SignerClient::from_env("NEONEXUS_SIGNER_ADMIN").map_err(|error| error.to_string())
            })
            .clone()
    }
}

impl Default for SignerHandle {
    fn default() -> Self {
        let resolved = OnceLock::new();
        let _ = resolved.set(Ok(None));
        Self {
            resolved: Arc::new(resolved),
        }
    }
}
