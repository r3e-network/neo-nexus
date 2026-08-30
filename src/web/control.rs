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
    core::{
        lifecycle::LaunchAction,
        operations::{
            AlertProvider, AlertRoutingPolicy, EventKind, EventSeverity, NewRuntimeEvent,
            RemoteFederationMonitorPolicy, RpcHealthMonitorPolicy,
        },
        runtime::RestartPolicy,
    },
    supervision,
    types::NodeConfig,
};

use super::{html, pages::settings, WebState};

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
