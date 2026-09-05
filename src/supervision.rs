//! The workbench's supervision engine: the loop that notices a node has died,
//! brings it back within policy, probes what the Settings page says to probe,
//! and routes the alerts the Alerts page says to route.
//!
//! Every one of those behaviours used to ride on the desktop shell's frame tick:
//! `src/app/frame.rs` drained probe results each frame, `rpc_health_flow` and
//! `remote_federation_flow` spawned probes on their policy intervals, and
//! `policy_alert_flow` delivered webhooks. Removing `src/app/` removed the
//! heartbeat but not the settings that describe it, so the workbench went on
//! offering policies that nothing executed and pages that implied they ran.
//!
//! Node launch and stop live here too, rather than being restated per frontend.
//! The CLI keeps its own thin wrapper because it must hand the process over on
//! exit; the browser and this loop share one code path and one supervisor.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    alerts::{deliver_webhook_alert, should_route_alert, AlertDeliveryStatus},
    config::ConfigExporter,
    core::{
        lifecycle::{execute_node_launch, LaunchAction},
        node::NodeConfig,
        operations::{evaluate_launch_readiness, evaluate_restart_readiness},
    },
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    federation::RemoteFederationClient,
    health_events::{
        exit_notice, exit_was_clean, remote_probe_event_severity, remote_probe_notice,
        rpc_health_event_severity, rpc_health_notice, should_record_remote_probe_event,
        should_record_rpc_health_event,
    },
    launch::LaunchPlanner,
    logs::LogReader,
    repository::Repository,
    rpc_health::probe_node_rpc,
    supervisor::{
        live_pids, log_path_for, recorded_process, PidStop, ProcessSupervisor, RecordedProcess,
    },
    types::NodeStatus,
    watchdog::{default_restart_policy, RestartOutcome, Watchdog},
};

mod recovery;
mod startup;

/// How often the loop wakes. Every interval it enforces is a multiple of this
/// or is compared against `Instant`, so a second keeps latency invisible while
/// leaving the tick cheap.
const TICK: Duration = Duration::from_secs(1);
const RPC_HEALTH_TIMEOUT: Duration = Duration::from_secs(3);
const FEDERATION_TIMEOUT: Duration = Duration::from_secs(5);
const RPC_HEALTH_RETAIN_PER_NODE: usize = crate::chain_progress::OBSERVATION_HISTORY_LIMIT;
const ALERT_DELIVERY_RETAIN: usize = 50;
const LOG_MAX_BYTES: usize = 64 * 1024;
const JOURNAL_SCAN_LIMIT: usize = 25;
/// How many due nodes one tick may probe, and how many journal events one tick
/// may offer to the alert route. Both bound the worst case of a tick — every
/// probe is a request that can wait out its full timeout, every delivery a
/// webhook that can do the same — so one dead endpoint or down webhook costs
/// the loop at most this much per second, and whatever exceeds the bound stays
/// due and is picked up on the next ticks.
const MAX_RPC_PROBES_PER_TICK: usize = 8;
const MAX_ALERT_DELIVERIES_PER_TICK: usize = 8;
/// How many ticks a failed webhook delivery is retried before the event is
/// given up. Every attempt is a row in the deliveries table an operator can
/// read; holding the cursor forever would silence the newer events behind it.
const ALERT_DELIVERY_MAX_ATTEMPTS: usize = 3;

/// Everything the engine needs, deliberately not called `WebState`: the
/// supervisor is shared with the browser, and the repository is opened per call
/// as everywhere else in the workspace.
#[derive(Clone)]
pub struct EngineState {
    pub repository: Repository,
    pub data_dir: PathBuf,
    pub supervisor: Arc<Mutex<ProcessSupervisor>>,
    /// Stamped by the loop every tick; /healthz turns it into a verdict. A
    /// guardian whose own liveness is invisible cannot be trusted end to end.
    pub heartbeat: crate::supervision_heartbeat::SupervisionHeartbeat,
}

impl EngineState {
    fn supervisor(&self) -> std::sync::MutexGuard<'_, ProcessSupervisor> {
        self.supervisor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn workspace_child_dir(&self, child: &str) -> PathBuf {
        self.data_dir.join(child)
    }

    fn nodes(&self) -> Vec<NodeConfig> {
        self.repository.list_nodes().unwrap_or_default()
    }

    fn journal(
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

/// Launch or restart a node through the shared pipeline: readiness first, then
/// managed config, then supervision, then status. Used by the browser and by
/// the watchdog, so an automatic restart cannot drift from a manual one.
pub fn launch_node(
    state: &EngineState,
    node: &NodeConfig,
    action: LaunchAction,
) -> anyhow::Result<String> {
    launch_node_guarded(state, node, action, || Ok(()))
}

fn action_kind(action: LaunchAction) -> &'static str {
    match action {
        LaunchAction::Start => "start",
        LaunchAction::Restart => "restart",
    }
}

/// Recheck a caller's authority after waiting for process control and before
/// writing configuration or signalling a process.
pub(crate) fn launch_node_guarded(
    state: &EngineState,
    node: &NodeConfig,
    action: LaunchAction,
    authorize: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<String> {
    let plugins = state.repository.list_plugin_states(&node.id)?;
    let work_dir = state.workspace_child_dir("nodes").join(&node.id);
    let managed_config_path = ConfigExporter::managed_target_path(&work_dir, node);
    let log_path = log_path_for(state.workspace_child_dir("logs"), node);

    let readiness = match action {
        LaunchAction::Start => evaluate_launch_readiness(
            node,
            std::slice::from_ref(node),
            &plugins,
            &managed_config_path,
            &work_dir,
        ),
        LaunchAction::Restart => evaluate_restart_readiness(
            node,
            std::slice::from_ref(node),
            &plugins,
            &managed_config_path,
            &work_dir,
        ),
    };
    if let Some(blocker) = readiness.blocking_summary() {
        anyhow::bail!("readiness blocked — {blocker}");
    }

    let plan = LaunchPlanner::plan(node, &managed_config_path, &work_dir);
    let mut supervisor = state.supervisor();
    if !state.nodes().iter().any(|current| current == node) {
        anyhow::bail!("node state changed while preparing launch; reload and retry");
    }
    authorize()?;
    // The operation ledger is what keeps a headless CLI and this engine from
    // interleaving on the same node: whoever loses this claim waits instead
    // of racing, and the guard closes the claim on every exit path.
    // Held for its Drop: the claim closes on every exit path below.
    let _operation = state
        .repository
        .begin_node_operation_guarded(action_kind(action), &node.id)?;
    if state.repository.list_plugin_states(&node.id)? != plugins {
        anyhow::bail!("plugin configuration changed while preparing launch; reload and retry");
    }
    // A restart stops by handle. If the running process came from an earlier
    // session, quiesce it by pid or this would start a second node on the same
    // ports.
    let replaced = action == LaunchAction::Restart
        && !supervisor.is_managing(&node.id)
        && crate::supervisor::recorded_process(node) == crate::supervisor::RecordedProcess::Alive;
    let outcome = execute_node_launch(
        &state.repository,
        &mut supervisor,
        node,
        &plan,
        &log_path,
        action,
        Some(crate::node_lifecycle::ManagedConfig {
            path: &managed_config_path,
            plugins: &plugins,
        }),
    );
    drop(supervisor);

    match outcome {
        crate::core::lifecycle::NodeLaunchOutcome::Started { pid, log_path } => {
            let message = format!(
                "{}{} launched with PID {}; log {}",
                if replaced {
                    "replaced an unmanaged process; "
                } else {
                    ""
                },
                node.name,
                pid,
                log_path.display()
            );
            // The journal records which control ran, so an operator asking
            // "who started this at 03:00" gets one answer rather than a gap.
            // A watchdog restart also lands here and adds its own entry above
            // this one, so the trigger and the effect are both visible.
            state.journal(
                node,
                match action {
                    LaunchAction::Start => EventKind::NodeStarted,
                    LaunchAction::Restart => EventKind::NodeRestarted,
                },
                EventSeverity::Info,
                message.clone(),
            );
            Ok(message)
        }
        crate::core::lifecycle::NodeLaunchOutcome::Failed { message } => {
            anyhow::bail!("{message}")
        }
    }
}

/// Stop a node, reaching the process by pid when this server holds no handle for
/// it. Marks the row stopped only after the process is confirmed gone or was
/// already absent.
pub fn stop_node(state: &EngineState, node: &NodeConfig) -> anyhow::Result<String> {
    stop_node_guarded(state, node, || Ok(()))
}

pub(crate) fn stop_node_guarded(
    state: &EngineState,
    node: &NodeConfig,
    authorize: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<String> {
    let log_path = log_path_for(state.workspace_child_dir("logs"), node);
    let mut supervisor = state.supervisor();
    if !state.nodes().iter().any(|current| current == node) {
        anyhow::bail!("node state changed before stop; reload and retry");
    }
    authorize()?;
    // Same ledger the launch path claims: a stop racing a start on the same
    // node from a different process is refused, not interleaved.
    // Held for its Drop: the claim closes on every exit path below.
    let _operation = state
        .repository
        .begin_node_operation_guarded("stop", &node.id)?;
    // Keep process control and the persisted status in the same critical section.
    let outcome = match supervisor.stop(&node.id)? {
        Some(stop) => PidStop::Stopped(stop),
        None => supervisor.stop_recorded_pid(node, &log_path)?,
    };
    match outcome {
        PidStop::Stopped(stop) => {
            state
                .repository
                .update_node_status(&node.id, NodeStatus::Stopped, None)?;
            let message = if stop.forced {
                format!("{} stopped (forced, pid {})", node.name, stop.pid)
            } else {
                format!("{} stopped (pid {})", node.name, stop.pid)
            };
            state.journal(
                node,
                EventKind::NodeStopped,
                EventSeverity::Info,
                message.clone(),
            );
            Ok(message)
        }
        PidStop::AlreadyGone => {
            state
                .repository
                .update_node_status(&node.id, NodeStatus::Stopped, None)?;
            Ok(format!("{} was not running", node.name))
        }
        // The number is held by something else now. We cannot know whether this
        // node is running, so nothing is signalled and no status is written.
        PidStop::PidReused => Err(anyhow::anyhow!(
            "pid {} belongs to a different process; {name} was left alone and its status unchanged",
            node.pid.unwrap_or_default(),
            name = node.name
        )),
    }
}

/// Handle to the running engine. Dropping it stops the loop and waits for the
/// thread, so a shutting-down server cannot leave a probe mid-flight.
pub struct Engine {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    heartbeat: crate::supervision_heartbeat::SupervisionHeartbeat,
}

impl Engine {
    pub fn start(state: EngineState) -> Self {
        // Before the first page can be served: a workspace reopened after a
        // crash still claims nodes are Running, and the operator should never
        // see a status the host does not back.
        let mut loop_state = LoopState::bootstrap(&state);
        loop_state.reconcile_startup(&state);
        let stop = Arc::new(AtomicBool::new(false));
        let closing = Arc::clone(&stop);
        let heartbeat = state.heartbeat.clone();
        // The first stamp is synchronous: a guardian that has begun reports
        // running even before its thread's first tick lands.
        heartbeat.beat();
        let loop_heartbeat = heartbeat.clone();
        let worker = thread::Builder::new()
            .name("neonexus-supervision".to_string())
            .spawn(move || {
                loop_heartbeat.beat();
                while !closing.load(Ordering::Relaxed) {
                    loop_state.tick(&state);
                    loop_heartbeat.beat();
                    thread::sleep(TICK);
                }
            });
        let worker = match worker {
            Ok(worker) => Some(worker),
            Err(error) => {
                // Used to be swallowed into `None`: the web process reported
                // healthy while nothing supervised the fleet.
                heartbeat.mark_failed(format!("supervision thread did not start: {error}"));
                None
            }
        };
        Self {
            stop,
            worker,
            heartbeat,
        }
    }

    /// The handle whose verdict /healthz reports.
    pub fn heartbeat(&self) -> crate::supervision_heartbeat::SupervisionHeartbeat {
        self.heartbeat.clone()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Monitor clocks and cached views of persisted operational state. Policy and
/// recovery changes are re-read so browser or CLI controls take effect next tick.
struct LoopState {
    signer: crate::signer_client::SignerMonitor,
    resources: crate::resource_health::ResourceMonitor,
    watchdog: Watchdog,
    recovery_error: Option<String>,
    rpc_last_probe: BTreeMap<String, Instant>,
    federation_last_probe: BTreeMap<String, Instant>,
    /// Highest journal id already offered to the alert route. New workspaces
    /// begin at the latest event; unreadable progress replays retained events.
    last_routed_event: i64,
    /// Delivery attempts per still-undelivered event. A failed webhook is
    /// retried on the next ticks up to [`ALERT_DELIVERY_MAX_ATTEMPTS`]; the
    /// map only ever holds the events between retries, so it stays tiny.
    alert_failures: BTreeMap<i64, usize>,
}

impl LoopState {
    fn bootstrap(state: &EngineState) -> Self {
        let policy = state
            .repository
            .load_watchdog_policy()
            .unwrap_or_else(|_| default_restart_policy());
        let (cursor, alert_failures) = startup::alert_progress(state);
        let mut engine = Self {
            signer: crate::signer_client::SignerMonitor::bootstrap(),
            resources: crate::resource_health::ResourceMonitor::default(),
            watchdog: Watchdog::new(policy),
            recovery_error: None,
            rpc_last_probe: BTreeMap::new(),
            federation_last_probe: BTreeMap::new(),
            last_routed_event: cursor,
            alert_failures,
        };
        engine.initialize_recovery(state);
        engine
    }

    fn tick(&mut self, state: &EngineState) {
        self.sync_policy(state);
        self.reconcile_exits(state);
        crate::agents::tick(state);
        self.run_due_restarts(state);
        self.watch_external_processes(state);
        self.signer.tick(state);
        self.probe_rpc_health(state);
        self.probe_federation(state);
        self.resources.tick(&state.repository);
        self.route_alerts(state);
    }

    fn sync_policy(&mut self, state: &EngineState) {
        self.refresh_recovery(state);
    }

    /// Take every process the supervisor was watching that has now finished,
    /// and decide what it means.
    fn reconcile_exits(&mut self, state: &EngineState) {
        let exits = match state.supervisor().reap_finished() {
            Ok(exits) => exits,
            Err(error) => {
                eprintln!("neo-nexus: failed to reap finished processes: {error}");
                return;
            }
        };
        if exits.is_empty() {
            return;
        }
        for exit in exits {
            if crate::agents::observe_exit(state, &exit) {
                continue;
            }
            self.reconcile_node_exit(state, &exit);
        }
    }

    fn reconcile_node_exit(&mut self, state: &EngineState, exit: &crate::supervisor::ProcessExit) {
        let supervisor = state.supervisor();
        let Some(node) = state
            .nodes()
            .into_iter()
            .find(|node| node.id == exit.node_id)
        else {
            return;
        };
        // Reaping releases the lock before observation. A new process may
        // already own this node, so an old exit must not clear its PID.
        if node.pid != Some(exit.pid) || supervisor.is_managing(&node.id) {
            return;
        }
        if exit_was_clean(exit) {
            self.watchdog.clear(&node.id);
            let _ = state
                .repository
                .update_node_status(&node.id, NodeStatus::Stopped, None);
            state.journal(
                &node,
                EventKind::NodeExited,
                EventSeverity::Info,
                exit_notice(&node.name, exit),
            );
            return;
        }
        let reason = self.exit_notice_with_log(&node, exit, &state.workspace_child_dir("logs"));
        self.schedule_restart(state, &node, &reason);
    }

    /// A crash message is worth more with the log's own diagnosis attached: the
    /// exit code says it failed, the log says why.
    fn exit_notice_with_log(
        &self,
        node: &NodeConfig,
        exit: &crate::supervisor::ProcessExit,
        log_dir: &PathBuf,
    ) -> String {
        let base = exit_notice(&node.name, exit);
        let Ok(snapshot) = LogReader::snapshot(log_path_for(log_dir, node), LOG_MAX_BYTES) else {
            return base;
        };
        let diagnosis = LogReader::diagnose(&snapshot);
        match diagnosis.status {
            crate::logs::LogDiagnosisStatus::Critical
            | crate::logs::LogDiagnosisStatus::Warning => {
                format!("{base}; log diagnosis: {}", diagnosis.summary)
            }
            _ => base,
        }
    }

    fn journal_restart(
        &self,
        state: &EngineState,
        node: &NodeConfig,
        reason: &str,
        outcome: RestartOutcome,
    ) {
        match outcome {
            RestartOutcome::Scheduled { attempt, delay } => state.journal(
                node,
                EventKind::WatchdogScheduled,
                EventSeverity::Warning,
                format!(
                    "{reason}; watchdog will retry in {}s (attempt {attempt})",
                    delay.as_secs()
                ),
            ),
            RestartOutcome::Exhausted { attempts } => state.journal(
                node,
                EventKind::WatchdogExhausted,
                EventSeverity::Critical,
                format!("{reason}; watchdog gave up after {attempts} attempts"),
            ),
            RestartOutcome::Disabled => state.journal(
                node,
                EventKind::NodeExited,
                EventSeverity::Warning,
                format!("{reason}; automatic restart is off"),
            ),
        }
    }

    /// Nodes recorded Running that this server holds no handle for — started by
    /// the CLI, or left alive across a restart — are watched by pid, so their
    /// status cannot stay true after the process is gone.
    fn watch_external_processes(&mut self, state: &EngineState) {
        let supervisor = state.supervisor();
        let candidates: Vec<(NodeConfig, u32)> = state
            .nodes()
            .into_iter()
            .filter(|node| node.status.is_running())
            .filter(|node| node.pid.is_some())
            .filter(|node| !supervisor.is_managing(&node.id))
            .filter_map(|node| node.pid.map(|pid| (node, pid)))
            .collect();
        drop(supervisor);
        if candidates.is_empty() {
            return;
        }
        // One pass over the process table for the whole tick, not one per node.
        let alive = live_pids(&candidates.iter().map(|(_, pid)| *pid).collect::<Vec<_>>());
        for (node, pid) in candidates {
            let supervisor = state.supervisor();
            // A concurrent browser stop/restart may already have replaced this PID.
            if supervisor.is_managing(&node.id)
                || !state.nodes().iter().any(|current| {
                    current.id == node.id && current.pid == Some(pid) && current.status.is_running()
                })
            {
                continue;
            }
            let identity = if alive.contains(&pid) {
                recorded_process(&node)
            } else {
                RecordedProcess::Gone
            };
            if identity == RecordedProcess::Alive {
                continue;
            }
            if identity == RecordedProcess::Reused {
                let _ = state
                    .repository
                    .update_node_status(&node.id, NodeStatus::Error, Some(pid));
                state.journal(&node, EventKind::NodeExited, EventSeverity::Critical,
                    format!("{} recorded PID {pid} belongs to another executable; automatic restart blocked", node.name));
                continue;
            }
            state.journal(
                &node,
                EventKind::NodeExited,
                EventSeverity::Critical,
                format!(
                    "{} is no longer running (pid {pid}); it was not supervised by this server",
                    node.name
                ),
            );
            self.schedule_restart(
                state,
                &node,
                "recorded process disappeared; exit code unavailable",
            );
        }
    }

    /// Whether something last done at `seen` is due again. Never having done it
    /// counts as due.
    fn due(&self, seen: Option<Instant>, now: Instant, interval: Duration) -> bool {
        seen.is_none_or(|seen| now.duration_since(seen) >= interval)
    }

    fn probe_rpc_health(&mut self, state: &EngineState) {
        let Ok(policy) = state.repository.load_rpc_health_monitor_policy() else {
            return;
        };
        if !policy.enabled {
            return;
        }
        let interval = policy.interval_duration();
        let now = Instant::now();
        let mut due: Vec<NodeConfig> = state
            .nodes()
            .into_iter()
            .filter(|node| {
                node.status.is_running()
                    && self.due(self.rpc_last_probe.get(&node.id).copied(), now, interval)
            })
            .collect();
        // Oldest/unprobed first: slow endpoints must not starve the tail of a fleet.
        due.sort_by_key(|node| self.rpc_last_probe.get(&node.id).copied());
        let reports = thread::scope(|scope| {
            let jobs: Vec<_> = due
                .into_iter()
                .take(MAX_RPC_PROBES_PER_TICK)
                .map(|node| {
                    scope.spawn(move || {
                        let report = probe_node_rpc(&node, RPC_HEALTH_TIMEOUT);
                        (node, report)
                    })
                })
                .collect();
            jobs.into_iter()
                .filter_map(|job| job.join().ok())
                .collect::<Vec<_>>()
        });
        if reports.is_empty() {
            return;
        }
        for (node, report) in reports {
            self.record_rpc_health(state, node, report, now);
        }
        // Evaluate the batch once. Its transaction rechecks node and observation
        // identity, so concurrent stops or fresh probes cannot commit stale alarms.
        if let Ok(elapsed) = SystemTime::now().duration_since(UNIX_EPOCH) {
            if let Err(error) = crate::chain_progress::check(&state.repository, elapsed.as_secs()) {
                eprintln!("neo-nexus: chain progress check failed: {error}");
            }
        }
    }

    fn record_rpc_health(
        &mut self,
        state: &EngineState,
        node: NodeConfig,
        report: crate::rpc_health::RpcHealthReport,
        now: Instant,
    ) {
        let _supervisor = state.supervisor();
        if !state.nodes().iter().any(|current| current == &node) {
            self.rpc_last_probe.remove(&node.id);
            return;
        }

        let previous = state.repository.latest_rpc_health(&node.id).ok().flatten();
        if state.repository.record_rpc_health(&node, &report).is_err() {
            return;
        }
        self.rpc_last_probe.insert(node.id.clone(), now);
        let _ = state
            .repository
            .prune_rpc_health_keep_recent_per_node(RPC_HEALTH_RETAIN_PER_NODE);
        let identity_changed = previous
            .as_ref()
            .is_some_and(|old| old.network.identity_status() != report.network.identity_status());
        if should_record_rpc_health_event(previous.as_ref().map(|old| old.status), report.status)
            || identity_changed
        {
            let message = rpc_health_notice(&report);
            state.journal(
                &node,
                EventKind::RpcHealthChecked,
                if report.network.identity_status()
                    == crate::rpc_health::RpcIdentityStatus::Mismatch
                {
                    EventSeverity::Critical
                } else {
                    rpc_health_event_severity(report.status)
                },
                format!("Automatic RPC health: {message}"),
            );
        }
    }

    fn probe_federation(&mut self, state: &EngineState) {
        let Ok(policy) = state.repository.load_remote_federation_monitor_policy() else {
            return;
        };
        if !policy.enabled {
            return;
        }
        let interval = policy.interval_duration();
        let now = Instant::now();
        let Ok(mut profiles) = state.repository.list_remote_servers() else {
            return;
        };
        profiles.sort_by_key(|profile| self.federation_last_probe.get(&profile.id).copied());
        let Some(profile) = profiles.into_iter().find(|profile| {
            profile.enabled
                && self.due(
                    self.federation_last_probe.get(&profile.id).copied(),
                    now,
                    interval,
                )
        }) else {
            return;
        };
        self.federation_last_probe.insert(profile.id.clone(), now);

        let report = match RemoteFederationClient::probe(&profile, FEDERATION_TIMEOUT) {
            Ok(report) => report,
            Err(error) => {
                eprintln!(
                    "neo-nexus: federation probe for {} failed: {error}",
                    profile.name
                );
                return;
            }
        };
        let previous = state
            .repository
            .latest_remote_server_probe(&profile.id)
            .ok()
            .flatten()
            .map(|record| record.status);
        if state
            .repository
            .record_remote_server_probe(&report)
            .is_err()
        {
            return;
        }
        if should_record_remote_probe_event(previous, report.status) {
            let message = remote_probe_notice(&profile.name, report.status, &report.message);
            let _ = state.repository.record_event(NewRuntimeEvent {
                node_id: None,
                node_name: None,
                kind: EventKind::RemoteServerProbed,
                severity: remote_probe_event_severity(report.status),
                message,
            });
        }
    }

    /// Offer anything new since the last scan to the configured alert route.
    /// Up to [`MAX_ALERT_DELIVERIES_PER_TICK`] deliveries per tick, oldest
    /// first, so a burst of events drains instead of queuing behind one-per-
    /// second. A failed delivery is retried on later ticks up to
    /// [`ALERT_DELIVERY_MAX_ATTEMPTS`] attempts — each one a row in the
    /// deliveries table — and the first failure ends the tick, so a webhook
    /// that just went down is not hammered for the whole batch. The journal
    /// keeps the backlog visible throughout.
    fn route_alerts(&mut self, state: &EngineState) {
        let Ok(policy) = state.repository.load_alert_routing_policy() else {
            return;
        };
        let Ok(events) = state
            .repository
            .list_events_after(self.last_routed_event, JOURNAL_SCAN_LIMIT)
        else {
            return;
        };
        if events.is_empty() {
            return;
        }

        for event in events.into_iter().take(MAX_ALERT_DELIVERIES_PER_TICK) {
            if !should_route_alert(&policy, &event) {
                self.alert_failures.remove(&event.id);
                self.last_routed_event = event.id;
                self.persist_alert_progress(state);
                continue;
            }
            let report = deliver_webhook_alert(&policy, &event, env!("CARGO_PKG_VERSION"));
            if state.repository.record_alert_delivery(&report).is_err() {
                return;
            }
            if report.status != AlertDeliveryStatus::Failed {
                self.alert_failures.remove(&event.id);
                self.last_routed_event = event.id;
                self.persist_alert_progress(state);
                continue;
            }
            let attempts = self.alert_failures.entry(event.id).or_insert(0);
            *attempts += 1;
            if *attempts >= ALERT_DELIVERY_MAX_ATTEMPTS {
                // Given up: every attempt is on record in the deliveries
                // table, and holding the cursor here would silence every
                // newer event behind a webhook that is down.
                self.alert_failures.remove(&event.id);
                self.last_routed_event = event.id;
            } else {
                // Wind the cursor back so this exact event is retried first
                // on the next tick, and stop delivering for this tick — a
                // webhook that just failed does not need hammering.
                self.last_routed_event = event.id - 1;
                self.persist_alert_progress(state);
                break;
            }
            self.persist_alert_progress(state);
        }
        let _ = state
            .repository
            .prune_alert_deliveries_keep_recent(ALERT_DELIVERY_RETAIN);
        // A failed delivery is recorded in the deliveries table, which the
        // Alerts page already renders; the journal is for state changes.
    }

    fn persist_alert_progress(&self, state: &EngineState) {
        if let Err(error) = state
            .repository
            .save_alert_progress(self.last_routed_event, &self.alert_failures)
        {
            eprintln!("neo-nexus: cannot persist alert progress: {error}");
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/supervision/tests.rs"]
mod tests;
