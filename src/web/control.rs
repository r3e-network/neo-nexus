//! Node lifecycle from the browser: the same core pipeline the CLI and the
//! former GUI share — readiness evaluation, managed config export, supervised
//! launch — plus the supervisor-backed stop. Handlers answer with a redirect
//! back to the node page carrying a flash message, so plain form posts work.

use std::time::Duration;

use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    core::{
        lifecycle::{execute_node_launch, LaunchAction, ManagedConfig, NodeLaunchOutcome},
        operations::{
            evaluate_launch_readiness, evaluate_restart_readiness, AlertProvider,
            AlertRoutingPolicy, EventSeverity, RemoteFederationMonitorPolicy,
            RpcHealthMonitorPolicy,
        },
        runtime::RestartPolicy,
        workspace::ConfigExporter,
    },
    launch::LaunchPlanner,
    supervisor::{log_path_for, ProcessSupervisor},
    types::{NodeConfig, NodeStatus},
};

use super::{html, pages::settings, WebState};

pub async fn node_start(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    control_redirect(&state, &id, LaunchAction::Start)
}

pub async fn node_restart(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    control_redirect(&state, &id, LaunchAction::Restart)
}

pub async fn node_stop(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let repository = &state.repository;
    let outcome = load_node(repository, &id).and_then(|node| stop_node(repository, &node));
    match outcome {
        Ok(message) => back_to_node(&id, &message),
        Err(error) => back_to_node(&id, &format!("stop failed: {error}")),
    }
}

fn control_redirect(state: &WebState, id: &str, action: LaunchAction) -> Response {
    let outcome =
        load_node(&state.repository, id).and_then(|node| launch_node(state, &node, action));
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

fn stop_node(
    repository: &crate::repository::Repository,
    node: &NodeConfig,
) -> anyhow::Result<String> {
    let mut supervisor = ProcessSupervisor::default();
    let stopped = supervisor.stop(&node.id)?;
    repository.update_node_status(&node.id, NodeStatus::Stopped, None)?;
    Ok(match stopped {
        Some(_) => format!("{} stopped", node.name),
        None => format!("{} was not running", node.name),
    })
}

fn launch_node(
    state: &WebState,
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
    let mut supervisor = ProcessSupervisor::default();
    let outcome = execute_node_launch(
        &state.repository,
        &mut supervisor,
        node,
        &plan,
        &log_path,
        action,
        Some(ManagedConfig {
            path: &managed_config_path,
            plugins: &plugins,
        }),
    );
    Ok(match outcome {
        NodeLaunchOutcome::Started { pid, log_path } => {
            format!(
                "{} launched with PID {}; log {}",
                node.name,
                pid,
                log_path.display()
            )
        }
        NodeLaunchOutcome::Failed { message } => {
            format!("{} launch failed: {message}", node.name)
        }
    })
}

/// The Settings page posts whole numbers as text, so a hand-edited form has to
/// be rejected rather than silently saved as zero.
fn whole_number(raw: &str, field: &str) -> anyhow::Result<u64> {
    raw.trim()
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("{field} must be a whole number of seconds"))
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
