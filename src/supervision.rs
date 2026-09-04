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
    time::{Duration, Instant},
};

use crate::{
    alerts::{deliver_webhook_alert, should_route_alert, AlertDeliveryStatus},
    config::ConfigExporter,
    core::{
        lifecycle::{execute_node_launch, LaunchAction},
        node::NodeConfig,
        operations::{evaluate_launch_readiness, evaluate_restart_readiness},
    },
    events::{EventKind, EventSeverity, NewRuntimeEvent, RuntimeEventFilter},
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
    watchdog::{default_restart_policy, RestartOutcome, RestartPolicy, Watchdog},
};

/// How often the loop wakes. Every interval it enforces is a multiple of this
/// or is compared against `Instant`, so a second keeps latency invisible while
/// leaving the tick cheap.
const TICK: Duration = Duration::from_secs(1);
const RPC_HEALTH_TIMEOUT: Duration = Duration::from_secs(3);
const FEDERATION_TIMEOUT: Duration = Duration::from_secs(5);
const RPC_HEALTH_RETAIN_PER_NODE: usize = 24;
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
    // A restart stops by handle. If the running process came from an earlier
    // session, quiesce it by pid or this would start a second node on the same
    // ports.
    let replaced = action == LaunchAction::Restart
        && crate::node_lifecycle::quiesce_before_restart(&mut supervisor, node, &log_path);
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
    let log_path = log_path_for(state.workspace_child_dir("logs"), node);
    let outcome = {
        let mut supervisor = state.supervisor();
        match supervisor.stop(&node.id)? {
            Some(stop) => PidStop::Stopped(stop),
            None => supervisor.stop_recorded_pid(node, &log_path),
        }
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
}

impl Engine {
    pub fn start(state: EngineState) -> Self {
        // Before the first page can be served: a workspace reopened after a
        // crash still claims nodes are Running, and the operator should never
        // see a status the host does not back.
        reconcile_startup(&state);
        let stop = Arc::new(AtomicBool::new(false));
        let closing = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("neonexus-supervision".to_string())
            .spawn(move || {
                let mut loop_state = LoopState::bootstrap(&state);
                while !closing.load(Ordering::Relaxed) {
                    loop_state.tick(&state);
                    thread::sleep(TICK);
                }
            });
        Self {
            stop,
            worker: worker.ok(),
        }
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

/// Settle nodes whose recorded process no longer exists, and keep the ones that
/// do.
///
/// Deliberately not a blanket "mark everything stopped": a workbench killed with
/// SIGKILL leaves its nodes running as orphans. Clearing those rows would lose
/// the only handle on them, and the next Start would launch a second node onto
/// the same ports. A node whose pid is answered by a *different* program is
/// settled too, but reported separately — the number was recycled, so the old
/// process is gone and something unrelated now holds its identity.
fn reconcile_startup(state: &EngineState) {
    let mut settled = Vec::new();
    let mut recycled = Vec::new();
    for node in state.nodes() {
        if !matches!(node.status, NodeStatus::Running | NodeStatus::Starting) {
            continue;
        }
        // One classification per node: each probe reads the process table.
        let verdict = recorded_process(&node);
        if verdict == RecordedProcess::Alive {
            continue;
        }
        let _ = state
            .repository
            .update_node_status(&node.id, NodeStatus::Stopped, None);
        match verdict {
            RecordedProcess::Reused => recycled.push(node.name),
            _ => settled.push(node.name),
        }
    }
    let total = settled.len() + recycled.len();
    if total == 0 {
        return;
    }
    let mut message = format!("Recovered {total} stale runtime state records");
    if !recycled.is_empty() {
        message.push_str(&format!(
            "; {} pid(s) now belong to another program",
            recycled.len()
        ));
    }
    let _ = state.repository.record_event(NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind: EventKind::RuntimeRecovered,
        severity: EventSeverity::Warning,
        message,
    });
}

/// What the loop remembers between ticks. Policies are re-read every tick so a
/// change made in Settings takes effect without a restart; these are the things
/// that cannot be re-derived from the database.
struct LoopState {
    watchdog: Watchdog,
    /// The policy the watchdog is running under, so a tick that reads an
    /// unchanged policy leaves scheduled restarts alone.
    applied_policy: RestartPolicy,
    rpc_last_probe: BTreeMap<String, Instant>,
    federation_last_probe: BTreeMap<String, Instant>,
    /// Highest journal id already offered to the alert route. Seeded at startup
    /// so starting the workbench cannot deliver a webhook for events from weeks
    /// ago.
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
        let newest = state
            .repository
            .list_events(RuntimeEventFilter::new(None, "", 1))
            .ok()
            .and_then(|events| events.first().map(|event| event.id));
        Self {
            watchdog: Watchdog::new(policy),
            applied_policy: policy,
            rpc_last_probe: BTreeMap::new(),
            federation_last_probe: BTreeMap::new(),
            last_routed_event: newest.unwrap_or_default(),
            alert_failures: BTreeMap::new(),
        }
    }

    fn tick(&mut self, state: &EngineState) {
        self.sync_policy(state);
        self.reconcile_exits(state);
        self.run_due_restarts(state);
        self.watch_external_processes(state);
        self.probe_rpc_health(state);
        self.probe_federation(state);
        self.route_alerts(state);
    }

    fn sync_policy(&mut self, state: &EngineState) {
        let Ok(policy) = state.repository.load_watchdog_policy() else {
            return;
        };
        // `update_policy` clears pending restarts, so pushing it on every tick
        // would wipe a scheduled retry before its delay ever elapsed.
        if policy != self.applied_policy {
            self.watchdog.update_policy(policy);
            self.applied_policy = policy;
        }
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
        let nodes = state.nodes();
        for exit in exits {
            let Some(node) = nodes.iter().find(|node| node.id == exit.node_id) else {
                continue;
            };
            if exit_was_clean(&exit) {
                self.watchdog.clear(&node.id);
                let _ = state
                    .repository
                    .update_node_status(&node.id, NodeStatus::Stopped, None);
                state.journal(
                    node,
                    EventKind::NodeExited,
                    EventSeverity::Info,
                    exit_notice(&node.name, &exit),
                );
                continue;
            }
            let reason = self.exit_notice_with_log(node, &exit, &state.workspace_child_dir("logs"));
            self.schedule_restart(state, node, &reason);
        }
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

    fn schedule_restart(&mut self, state: &EngineState, node: &NodeConfig, reason: &str) {
        // Crashed, not Error: a dirty exit and a launch that never got off the
        // ground are different failures an operator triages differently, and
        // one shared status made them indistinguishable.
        let _ = state
            .repository
            .update_node_status(&node.id, NodeStatus::Crashed, None);
        match self.watchdog.record_failure(&node.id, Instant::now()) {
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

    fn run_due_restarts(&mut self, state: &EngineState) {
        let due = self.watchdog.due_restarts(Instant::now());
        if due.is_empty() {
            return;
        }
        let nodes = state.nodes();
        for attempt in due {
            let Some(node) = nodes.iter().find(|node| node.id == attempt.node_id) else {
                self.watchdog.clear(&attempt.node_id);
                continue;
            };
            match launch_node(state, node, LaunchAction::Start) {
                Ok(message) => state.journal(
                    node,
                    EventKind::WatchdogRestarted,
                    EventSeverity::Warning,
                    format!("watchdog attempt {}: {message}", attempt.attempt),
                ),
                Err(error) => state.journal(
                    node,
                    EventKind::NodeStartFailed,
                    EventSeverity::Critical,
                    format!("watchdog attempt {} failed: {error}", attempt.attempt),
                ),
            }
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
            if alive.contains(&pid) {
                continue;
            }
            let _ = state
                .repository
                .update_node_status(&node.id, NodeStatus::Stopped, None);
            state.journal(
                &node,
                EventKind::NodeExited,
                EventSeverity::Warning,
                format!(
                    "{} is no longer running (pid {pid}); it was not supervised by this server",
                    node.name
                ),
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
        let due: Vec<NodeConfig> = state
            .nodes()
            .into_iter()
            .filter(|node| {
                node.status.is_running()
                    && self.due(self.rpc_last_probe.get(&node.id).copied(), now, interval)
            })
            .collect();
        // Every due node is probed, not one per tick: with more running nodes
        // than ticks in the policy interval, one-per-tick silently stretched
        // the real interval to fleet-size seconds.
        for node in due.into_iter().take(MAX_RPC_PROBES_PER_TICK) {
            self.probe_one_rpc_health(state, node, now);
        }
    }

    fn probe_one_rpc_health(&mut self, state: &EngineState, node: NodeConfig, now: Instant) {
        self.rpc_last_probe.insert(node.id.clone(), now);

        let report = probe_node_rpc(&node, RPC_HEALTH_TIMEOUT);
        let previous = state
            .repository
            .latest_rpc_health(&node.id)
            .ok()
            .flatten()
            .map(|record| record.status);
        if state.repository.record_rpc_health(&node, &report).is_err() {
            return;
        }
        let _ = state
            .repository
            .prune_rpc_health_keep_recent_per_node(RPC_HEALTH_RETAIN_PER_NODE);
        if should_record_rpc_health_event(previous, report.status) {
            let message = rpc_health_notice(&report);
            state.journal(
                &node,
                EventKind::RpcHealthChecked,
                rpc_health_event_severity(report.status),
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
        let Ok(profiles) = state.repository.list_remote_servers() else {
            return;
        };
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
        let Ok(events) =
            state
                .repository
                .list_events(RuntimeEventFilter::new(None, "", JOURNAL_SCAN_LIMIT))
        else {
            return;
        };
        let mut pending: Vec<_> = events
            .into_iter()
            .filter(|event| event.id > self.last_routed_event)
            .collect();
        if pending.is_empty() {
            return;
        }
        pending.sort_by_key(|event| event.id);

        for event in pending.into_iter().take(MAX_ALERT_DELIVERIES_PER_TICK) {
            if !should_route_alert(&policy, &event) {
                self.last_routed_event = event.id;
                continue;
            }
            let report = deliver_webhook_alert(&policy, &event, env!("CARGO_PKG_VERSION"));
            if state.repository.record_alert_delivery(&report).is_err() {
                return;
            }
            if report.status != AlertDeliveryStatus::Failed {
                self.alert_failures.remove(&event.id);
                self.last_routed_event = event.id;
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
                break;
            }
        }
        let _ = state
            .repository
            .prune_alert_deliveries_keep_recent(ALERT_DELIVERY_RETAIN);
        // A failed delivery is recorded in the deliveries table, which the
        // Alerts page already renders; the journal is for state changes.
    }
}
