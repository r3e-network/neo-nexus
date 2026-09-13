//! Settings and policy form save handlers.

use std::time::Duration;

use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::core::{
    operations::{
        AlertProvider, AlertRoutingPolicy, EventKind, EventSeverity, NewRuntimeEvent,
        RemoteFederationMonitorPolicy, RpcHealthMonitorPolicy,
    },
    runtime::{validate_runtime_upgrade_policy, RestartPolicy, RuntimeUpgradePolicy},
};

use super::super::{assets::DensityMode, html, pages::settings, WebState};

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
    #[serde(default)]
    jitter_enabled: Option<String>,
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

/// Save the UI density preference.
pub async fn save_density(
    State(state): State<WebState>,
    Form(input): Form<DensityForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let density = DensityMode::from_str(&input.ui_density);
        state.commands.save_app_ui_density(density.as_str())?;
        Ok(format!("UI density saved — {density}"))
    })();
    respond_to("/settings", outcome)
}

/// Save watchdog policy.
pub async fn save_watchdog(
    State(state): State<WebState>,
    Form(input): Form<WatchdogForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let attempts = whole_number(&input.max_restart_attempts, "max attempts")?;
        let base = whole_number(&input.base_delay_seconds, "base delay")?;
        let cap = whole_number(&input.max_delay_seconds, "max delay")?;
        let jitter = match input.jitter_enabled.as_deref() {
            None => false,
            Some(value) if value.eq_ignore_ascii_case("true") || value == "on" => true,
            Some(value) if value.eq_ignore_ascii_case("false") => false,
            Some(value) => anyhow::bail!("invalid jitter value: {value}"),
        };
        let policy = RestartPolicy::with_enabled(
            settings::choice_is_enabled(&input.enabled),
            u32::try_from(attempts).map_err(|_| anyhow::anyhow!("max attempts is out of range"))?,
            Duration::from_secs(base),
            Duration::from_secs(cap),
        )
        .with_jitter(jitter)
        .normalized();
        let message = format!("watchdog policy saved — {}", policy.describe());
        state.commands.save_watchdog_policy(policy)?;
        journal_policy(&state, EventKind::WatchdogPolicyUpdated, &message);
        Ok(message)
    })();
    respond_to("/settings", outcome)
}

/// Save RPC health monitor policy.
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
        state.commands.save_rpc_health_monitor_policy(policy)?;
        journal_policy(&state, EventKind::RpcHealthMonitorPolicyUpdated, &message);
        Ok(message)
    })();
    respond_to("/settings", outcome)
}

/// Save remote federation monitor policy.
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
            .commands
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

/// Save runtime upgrade policy.
pub async fn save_runtime_upgrade_policy(
    State(state): State<WebState>,
    Form(input): Form<RuntimeUpgradeForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let current = state.workspace.load_runtime_upgrade_policy()?;
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
        state.commands.save_runtime_upgrade_policy(&policy)?;
        journal_policy(&state, EventKind::RuntimeUpgradePolicyUpdated, &message);
        Ok(message)
    })();
    respond_to("/settings", outcome)
}

/// Save alert routing policy.
pub async fn save_alert_routing(
    State(state): State<WebState>,
    Form(input): Form<AlertRoutingForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let current = state.workspace.load_alert_routing_policy()?;
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
        state.commands.save_alert_routing_policy(policy)?;
        Ok(message)
    })();
    respond_to("/alerts", outcome)
}

fn whole_number(raw: &str, field: &str) -> anyhow::Result<u64> {
    raw.trim()
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("{field} must be a whole number"))
}

fn journal_policy(state: &WebState, kind: EventKind, message: &str) {
    let _ = state.commands.record_event(NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind,
        severity: EventSeverity::Info,
        message: message.to_string(),
    });
}

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
