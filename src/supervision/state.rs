//! What the loop is given, and what it remembers between ticks.
//!
//! `EngineState` is deliberately not called `WebState`: the supervisor is
//! shared with the browser, and the repository is opened per call as everywhere
//! else in the workspace. `LoopState` holds only what cannot be re-derived from
//! the database — policies are re-read every tick so a change made in Settings
//! takes effect without a restart.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    core::node::NodeConfig,
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    metrics::MetricsStore,
    repository::Repository,
    signing::SignerRegistry,
    supervisor::ProcessSupervisor,
    watchdog::{default_restart_policy, RestartPolicy, Watchdog},
};

/// Everything the engine needs to run one tick.
#[derive(Clone)]
pub struct EngineState {
    pub repository: Repository,
    pub data_dir: PathBuf,
    pub supervisor: Arc<Mutex<ProcessSupervisor>>,
    pub signer_registry: SignerRegistry,
    /// The same store the browser reads, so a page shows the reading the tick
    /// took rather than one it takes for itself. Two collectors would sample
    /// the host at unrelated moments and disagree about its load.
    pub metrics: Arc<MetricsStore>,
}

impl EngineState {
    pub(super) fn supervisor(&self) -> std::sync::MutexGuard<'_, ProcessSupervisor> {
        self.supervisor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn workspace_child_dir(&self, child: &str) -> PathBuf {
        self.data_dir.join(child)
    }

    pub(super) fn nodes(&self) -> Vec<NodeConfig> {
        self.repository.list_nodes().unwrap_or_default()
    }

    pub(super) fn journal(
        &self,
        node: &NodeConfig,
        kind: EventKind,
        severity: EventSeverity,
        message: String,
    ) {
        let _ = self.repository.record_event(NewRuntimeEvent {
            node_id: Some(node.id.clone()),
            node_name: Some(node.name.clone()),
            kind,
            severity,
            message,
        });
    }
}

/// What the loop remembers between ticks. Policies are re-read every tick so a
/// change made in Settings takes effect without a restart; these are the things
/// that cannot be re-derived from the database.
pub(super) struct LoopState {
    pub(super) watchdog: Watchdog,
    /// The policy the watchdog is running under, so a tick that reads an
    /// unchanged policy leaves scheduled restarts alone.
    pub(super) applied_policy: RestartPolicy,
    pub(super) federation_last_probe: BTreeMap<String, Instant>,
    /// Highest journal id already offered to the alert route. Seeded at startup
    /// so starting the workbench cannot deliver a webhook for events from weeks
    /// ago.
    pub(super) last_routed_event: i64,
}

impl LoopState {
    pub(super) fn bootstrap(state: &EngineState) -> Self {
        let policy = state
            .repository
            .load_watchdog_policy()
            .unwrap_or_else(|_| default_restart_policy());
        // Start the alert checkpoint at the current journal head so the loop only
        // routes events that occur after the engine starts. `latest_event_id`
        // reads MAX(id) directly, independent of imported or nonmonotonic timestamps.
        let newest = state.repository.latest_event_id().unwrap_or_default();
        Self {
            watchdog: Watchdog::new(policy),
            applied_policy: policy,
            federation_last_probe: BTreeMap::new(),
            last_routed_event: newest,
        }
    }

    pub(super) fn tick(&mut self, state: &EngineState) {
        self.sync_policy(state);
        self.reconcile_exits(state);
        self.run_due_restarts(state);
        self.watch_external_processes(state);
        // Cheap: the store decides whether a sample is due, and a reading that
        // is not due costs one comparison.
        state.metrics.refresh_if_due(&state.nodes());
        self.probe_federation(state);
        self.route_alerts(state);
        self.probe_runtime_upgrade(state);
    }
}
