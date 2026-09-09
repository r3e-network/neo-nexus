//! Controls from the browser. Node lifecycle delegates to the supervision
//! engine, so a start from the page and a start from the watchdog are the same
//! code path against the same supervisor; policy forms do the same for settings.
//! Handlers answer with a redirect carrying a flash message, so every control is
//! a plain form post that works without JavaScript.

use std::time::Duration;

use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    backup::workspace_exporter::WorkspaceBackupExporter,
    core::{
        lifecycle::LaunchAction,
        operations::{
            AlertProvider, AlertRoutingPolicy, EventKind, EventSeverity, NewRuntimeEvent,
            RemoteFederationMonitorPolicy, RpcHealthMonitorPolicy,
        },
        runtime::{validate_runtime_upgrade_policy, RestartPolicy, RuntimeUpgradePolicy},
    },
    snapshots::FastSyncSnapshotManager,
    supervision,
    types::NodeConfig,
};

use super::{assets::DensityMode, html, pages::settings, WebState};

use crate::runtime_smoke;

pub async fn node_start(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    control_redirect(&state, &id, LaunchAction::Start)
}

pub async fn node_restart(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    control_redirect(&state, &id, LaunchAction::Restart)
}

pub async fn node_stop(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let outcome = load_node(&state.repository, &id)
        .and_then(|node| supervision::stop_node(&state.engine_state(), &node));
    match outcome {
        Ok(message) => back_to_node(&id, &message),
        Err(error) => back_to_node(&id, &format!("stop failed: {error}")),
    }
}

pub async fn apply_snapshot(
    State(state): State<WebState>,
    Path((snapshot_id, node_id)): Path<(String, String)>,
) -> Response {
    use crate::types::node_workspace_path;

    let outcome = (|| -> anyhow::Result<String> {
        // Load the snapshot
        let snapshot = state
            .repository
            .list_fast_sync_snapshots()?
            .into_iter()
            .find(|s| s.id == snapshot_id)
            .ok_or_else(|| anyhow::anyhow!("snapshot {snapshot_id} was not found"))?;

        // Verify snapshot is ready to apply (has verified hash and cached file)
        if snapshot.verified_sha256.is_none() {
            anyhow::bail!("snapshot {snapshot_id} has not been hash-verified");
        }
        if snapshot.cached_path.is_none() {
            anyhow::bail!("snapshot {snapshot_id} has not been downloaded");
        }

        // Load the target node
        let node = load_node(&state.repository, &node_id)?;

        // Construct node data directory path
        let workspace_root = std::path::Path::new("workspaces"); // TODO: Make this configurable
        let node_workspace = node_workspace_path(workspace_root, &node.id)?;
        let node_data_dir = node_workspace
            .join("data")
            .join(snapshot.network.to_string());

        // Apply the snapshot
        let application = FastSyncSnapshotManager::apply_to_node(&snapshot, &node, &node_data_dir)?;

        let message = format!(
            "snapshot '{}' applied to node '{}' ({}) — {} files, {} decompressed",
            snapshot.label,
            node.id,
            node.name,
            application.imported_files,
            crate::core::operations::format_bytes(application.expanded_bytes),
        );

        // The apply already happened; a journal failure must not be reported as
        // if the apply itself had failed.
        let _ = state.repository.record_event(NewRuntimeEvent {
            node_id: Some(node.id.clone()),
            node_name: Some(node.name.clone()),
            kind: EventKind::SnapshotApplied,
            severity: EventSeverity::Info,
            message: message.clone(),
        });

        Ok(message)
    })();

    match outcome {
        Ok(message) => back_to_node(&node_id, &message),
        Err(error) => back_to_node(&node_id, &format!("failed: {error}")),
    }
}

fn control_redirect(state: &WebState, id: &str, action: LaunchAction) -> Response {
    let outcome = load_node(&state.repository, id)
        .and_then(|node| supervision::launch_node(&state.engine_state(), &node, action));
    match outcome {
        Ok(message) => back_to_node(id, &message),
        Err(error) => back_to_node(id, &format!("failed: {error}")),
    }
}

fn back_to_node(id: &str, message: &str) -> Response {
    Redirect::to(&format!(
        "/nodes/{}?flash={}",
        html::urlencoding_lite(id),
        html::urlencoding_lite(message),
    ))
    .into_response()
}

fn load_node(repository: &crate::repository::Repository, id: &str) -> anyhow::Result<NodeConfig> {
    repository
        .list_nodes()?
        .into_iter()
        .find(|node| node.id == id)
        .ok_or_else(|| anyhow::anyhow!("node {id} was not found"))
}

/// Policy forms post whole numbers as text, so a hand-edited submission has to
/// be refused rather than silently saved as zero.
fn whole_number(raw: &str, field: &str) -> anyhow::Result<u64> {
    raw.trim()
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("{field} must be a whole number"))
}

#[derive(Deserialize)]
pub struct WatchdogForm {
    #[serde(default)]
    enabled: String,
    #[serde(default)]
    max_restart_attempts: String,
    #[serde(default)]
    base_delay_seconds: String,
    #[serde(default)]
    max_delay_seconds: String,
}

#[derive(Deserialize)]
pub struct MonitorForm {
    #[serde(default)]
    enabled: String,
    #[serde(default)]
    interval_seconds: String,
}

#[derive(Deserialize)]
pub struct DensityForm {
    #[serde(default)]
    ui_density: String,
}

/// Save the UI density preference. The submitted label is normalised through
/// [`DensityMode`] so a hand-edited post can only ever persist a value the
/// workbench recognises — anything unknown falls back to comfortable.
pub async fn save_density(
    State(state): State<WebState>,
    Form(input): Form<DensityForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let density = DensityMode::from_str(&input.ui_density);
        state.repository.save_app_ui_density(density.as_str())?;
        Ok(format!("UI density saved — {density}"))
    })();
    respond_to("/settings", outcome)
}

pub async fn save_watchdog(
    State(state): State<WebState>,
    Form(input): Form<WatchdogForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let attempts = whole_number(&input.max_restart_attempts, "max attempts")?;
        let base = whole_number(&input.base_delay_seconds, "base delay")?;
        let cap = whole_number(&input.max_delay_seconds, "max delay")?;
        let policy = RestartPolicy::with_enabled(
            settings::choice_is_enabled(&input.enabled),
            u32::try_from(attempts).map_err(|_| anyhow::anyhow!("max attempts is out of range"))?,
            Duration::from_secs(base),
            Duration::from_secs(cap),
        )
        .normalized();
        let message = format!("watchdog policy saved — {}", policy.describe());
        state.repository.save_watchdog_policy(policy)?;
        journal_policy(&state, EventKind::WatchdogPolicyUpdated, &message);
        Ok(message)
    })();
    respond_to("/settings", outcome)
}

pub async fn save_rpc_health_monitor(
    State(state): State<WebState>,
    Form(input): Form<MonitorForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let interval = whole_number(&input.interval_seconds, "interval")?;
        let policy = RpcHealthMonitorPolicy {
            enabled: settings::choice_is_enabled(&input.enabled),
            interval_seconds: interval,
        }
        .normalized();
        let message = format!("RPC health monitor saved — {}", policy.describe());
        state.repository.save_rpc_health_monitor_policy(policy)?;
        journal_policy(&state, EventKind::RpcHealthMonitorPolicyUpdated, &message);
        Ok(message)
    })();
    respond_to("/settings", outcome)
}

pub async fn save_federation_monitor(
    State(state): State<WebState>,
    Form(input): Form<MonitorForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let interval = whole_number(&input.interval_seconds, "interval")?;
        let policy = RemoteFederationMonitorPolicy {
            enabled: settings::choice_is_enabled(&input.enabled),
            interval_seconds: interval,
        }
        .normalized();
        let message = format!("federation monitor saved — {}", policy.describe());
        state
            .repository
            .save_remote_federation_monitor_policy(policy)?;
        journal_policy(
            &state,
            EventKind::RemoteFederationMonitorPolicyUpdated,
            &message,
        );
        Ok(message)
    })();
    respond_to("/settings", outcome)
}

#[derive(Deserialize)]
pub struct RuntimeUpgradeForm {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    catalog_profile_id: String,
    #[serde(default)]
    interval_minutes: String,
    #[serde(default)]
    require_signed_catalog: Option<bool>,
    #[serde(default)]
    max_nodes_per_run: String,
    #[serde(default)]
    maintenance_window_enabled: Option<bool>,
    #[serde(default)]
    maintenance_window_start_minute_utc: String,
    #[serde(default)]
    maintenance_window_end_minute_utc: String,
    #[serde(default)]
    wave_delay_minutes: String,
}

/// Save the runtime upgrade policy. The two timestamp fields are the engine's
/// own record of what it did, so they never appear in the form — the current
/// values are loaded and carried across, leaving only the operator-editable
/// fields to the submission.
pub async fn save_runtime_upgrade_policy(
    State(state): State<WebState>,
    Form(input): Form<RuntimeUpgradeForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let current = state.repository.load_runtime_upgrade_policy()?;
        let interval_minutes = whole_number(&input.interval_minutes, "interval")?;
        let max_nodes = whole_number(&input.max_nodes_per_run, "nodes per run")?;
        let window_start =
            whole_number(&input.maintenance_window_start_minute_utc, "window start")?;
        let window_end = whole_number(&input.maintenance_window_end_minute_utc, "window end")?;
        let wave_delay = whole_number(&input.wave_delay_minutes, "wave delay")?;
        let profile = input.catalog_profile_id.trim();
        let policy = RuntimeUpgradePolicy {
            enabled: input.enabled.unwrap_or(false),
            catalog_profile_id: if profile.is_empty() {
                None
            } else {
                Some(profile.to_string())
            },
            interval_minutes,
            require_signed_catalog: input.require_signed_catalog.unwrap_or(false),
            max_nodes_per_run: usize::try_from(max_nodes)
                .map_err(|_| anyhow::anyhow!("nodes per run is out of range"))?,
            maintenance_window_enabled: input.maintenance_window_enabled.unwrap_or(false),
            maintenance_window_start_minute_utc: u16::try_from(window_start)
                .map_err(|_| anyhow::anyhow!("window start is out of range"))?,
            maintenance_window_end_minute_utc: u16::try_from(window_end)
                .map_err(|_| anyhow::anyhow!("window end is out of range"))?,
            wave_delay_minutes: wave_delay,
            last_checked_at_unix: current.last_checked_at_unix,
            last_applied_at_unix: current.last_applied_at_unix,
        };
        validate_runtime_upgrade_policy(&policy)?;
        let message = format!(
            "runtime upgrade policy saved \u{2014} {}",
            policy.describe()
        );
        state.repository.save_runtime_upgrade_policy(&policy)?;
        journal_policy(&state, EventKind::RuntimeUpgradePolicyUpdated, &message);
        Ok(message)
    })();
    respond_to("/settings", outcome)
}

#[derive(Deserialize)]
pub struct AlertRoutingForm {
    #[serde(default)]
    enabled: String,
    #[serde(default)]
    provider: String,
    #[serde(default)]
    min_severity: String,
    #[serde(default)]
    webhook_url: String,
    #[serde(default)]
    timeout_seconds: String,
}

/// Save the alert routing policy. A blank webhook keeps the stored target, so
/// the page never has to echo a provider token back to the browser.
pub async fn save_alert_routing(
    State(state): State<WebState>,
    Form(input): Form<AlertRoutingForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let current = state.repository.load_alert_routing_policy()?;
        let provider: AlertProvider = input
            .provider
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("{} is not a provider", input.provider))?;
        let min_severity: EventSeverity = input
            .min_severity
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("{} is not a severity", input.min_severity))?;
        let timeout_seconds = whole_number(&input.timeout_seconds, "timeout")?;
        let submitted = input.webhook_url.trim();
        let policy = AlertRoutingPolicy {
            enabled: settings::choice_is_enabled(&input.enabled),
            provider,
            min_severity,
            webhook_url: if submitted.is_empty() {
                current.webhook_url
            } else {
                Some(submitted.to_string())
            },
            timeout_seconds,
        }
        .normalized();
        if let Some(problem) = policy.validation_message() {
            anyhow::bail!("{problem}");
        }
        let message = format!("alert routing saved — {}", policy.describe());
        state.repository.save_alert_routing_policy(policy)?;
        Ok(message)
    })();
    respond_to("/alerts", outcome)
}

/// Export workspace backup archive. All artifacts — profiles, nodes, snapshots,
/// events — are captured and written to the workspace export directory, with a
/// summary returned as a flash message.
pub async fn handle_backup_export(State(state): State<WebState>) -> Response {
    use crate::core::operations::EventKind;

    let outcome = (|| -> anyhow::Result<String> {
        let output = state.workspace_child_dir("export").join("backup");
        let export =
            WorkspaceBackupExporter::write(&state.repository, &output, env!("CARGO_PKG_VERSION"))?;

        // Calculate total profile count
        let total_profiles = export.remote_server_count
            + export.runtime_catalog_profile_count
            + export.runtime_signer_profile_count
            + export.neo_wallet_profile_count;

        let message = format!(
            "workspace backup exported — {} nodes, {} profiles, {} snapshots, {} events → {}",
            export.node_count,
            total_profiles,
            export.fast_sync_snapshot_count,
            export.event_count,
            export.path.display(),
        );
        let _ = state.repository.record_event(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::BackupExported,
            severity: EventSeverity::Info,
            message: message.clone(),
        });
        Ok(message)
    })();

    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("backup export failed: {error}"),
    };
    Redirect::to(&format!(
        "/backup?flash={}",
        html::urlencoding_lite(&message),
    ))
    .into_response()
}

/// Record a workspace-level change. Policies are not tied to a node, so the
/// entry carries no node reference — but it must exist, or "when did this
/// change and to what?" has no answer.
fn journal_policy(state: &WebState, kind: EventKind, message: &str) {
    let _ = state.repository.record_event(NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind,
        severity: EventSeverity::Info,
        message: message.to_string(),
    });
}

/// Clear all log files from the workspace logs directory. This is a destructive
/// operation requiring confirmation via POST, and records an audit event on
/// successful completion.
pub async fn clear_logs(State(state): State<WebState>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        // Get the logs directory
        let logs_dir = state.workspace_child_dir("logs");

        // Clear all log files
        let cleared_count = crate::logs::clear_all_logs(&logs_dir)?;

        // Record audit event
        let message = format!("cleared {} log file(s)", cleared_count);
        let _ = state.repository.record_event(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::LogCleared,
            severity: EventSeverity::Info,
            message: message.clone(),
        });

        Ok(message)
    })();

    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("failed to clear logs: {error}"),
    };

    Redirect::to(&format!("/logs?flash={}", html::urlencoding_lite(&message))).into_response()
}

/// The shared tail of every settings-style control: describe the outcome and
/// send the browser back to the page that owns it.
fn respond_to(path: &str, outcome: anyhow::Result<String>) -> Response {
    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("not saved: {error}"),
    };
    Redirect::to(&format!(
        "{path}?flash={}",
        html::urlencoding_lite(&message),
        path = path
    ))
    .into_response()
}

/// Run a runtime smoke test against a node's current binary.
pub async fn smoke_test_node(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    use std::time::Duration;

    // First load the node synchronously
    let node = match load_node(&state.repository, &id) {
        Ok(n) => n,
        Err(error) => return back_to_node(&id, &format!("node not found: {error}")),
    };

    // Run smoke test in a blocking thread since it spawns processes
    let report = match tokio::task::spawn_blocking({
        let node_config = node.clone();
        move || runtime_smoke::smoke_node_binary(&node_config, Duration::from_secs(3))
    })
    .await
    .map_err(|e| anyhow::anyhow!("smoke test task failed: {e}"))
    {
        Ok(r) => r,
        Err(e) => {
            return back_to_node(&id, &format!("smoke test failed: {e}"));
        }
    };

    let message = format!(
        "runtime smoke test {} — {}",
        if report.status.is_success() {
            "passed"
        } else {
            "failed"
        },
        report.message
    );

    // Record event with severity matching status
    let _ = state.repository.record_event(NewRuntimeEvent {
        node_id: Some(id.clone()),
        node_name: Some(node.name.clone()),
        kind: EventKind::RuntimeSmokeTested,
        severity: if report.status.is_success() {
            EventSeverity::Info
        } else {
            EventSeverity::Warning
        },
        message: message.clone(),
    });

    back_to_node(&id, &message)
}
