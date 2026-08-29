//! Settings: the workspace policies that have no node of their own — the
//! watchdog restart budget and the two monitor intervals. Each form posts to a
//! handler that rebuilds the policy through the domain's own `normalized()`, so
//! the bounds live in one place and the page cannot invent a wider range.

use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse, Response},
};

use crate::{
    core::{
        operations::{RemoteFederationMonitorPolicy, RpcHealthMonitorPolicy},
        runtime::{RestartPolicy, RuntimeUpgradePolicy},
    },
    repository::Repository,
};

use super::super::{html, WebState};

const ENABLED_CHOICES: &[&str] = &["Enabled", "Disabled"];

pub async fn settings(State(state): State<WebState>, RawQuery(query): RawQuery) -> Response {
    let body = match render_body(&state.repository) {
        Ok(body) => body,
        Err(error) => html::note(&format!("failed to load settings: {error}")),
    };
    Html(html::layout(
        "Settings",
        "settings",
        &html::flash(query.as_deref()),
        &body,
    ))
    .into_response()
}

fn render_body(repository: &Repository) -> anyhow::Result<String> {
    let watchdog = repository.load_watchdog_policy()?;
    let rpc_health = repository.load_rpc_health_monitor_policy()?;
    let federation = repository.load_remote_federation_monitor_policy()?;
    let upgrade = repository.load_runtime_upgrade_policy()?;
    Ok(format!(
        r#"<h1>Settings</h1>
{engine_note}
{watchdog}
{rpc_health}
{federation}
<h2>Runtime upgrades</h2>
{upgrade}"#,
        engine_note = html::notice(
            "ok",
            "Applied by this workbench process: the supervision engine reads these on its own tick, so a saved change takes effect without a restart.",
        ),
        watchdog = watchdog_form(&watchdog),
        rpc_health = monitor_form(
            "RPC health monitor",
            "/settings/rpc-health",
            &rpc_health.describe(),
            rpc_health.enabled,
            rpc_health.interval_seconds,
            RpcHealthMonitorPolicy::MIN_INTERVAL_SECONDS,
            RpcHealthMonitorPolicy::MAX_INTERVAL_SECONDS,
        ),
        federation = monitor_form(
            "Federation monitor",
            "/settings/federation",
            &federation.describe(),
            federation.enabled,
            federation.interval_seconds,
            RemoteFederationMonitorPolicy::MIN_INTERVAL_SECONDS,
            RemoteFederationMonitorPolicy::MAX_INTERVAL_SECONDS,
        ),
        upgrade = upgrade_facts(&upgrade),
    ))
}

fn watchdog_form(policy: &RestartPolicy) -> String {
    format!(
        r#"<h2>Watchdog restarts</h2>
<p class="muted">{describe}</p>
<form class="filters" method="post" action="/settings/watchdog">
{enabled}
{attempts}
{base}
{cap}
<button type="submit">Save</button>
</form>"#,
        describe = html::escape(&policy.describe()),
        enabled = html::choice_field(
            "Status",
            "enabled",
            &enabled_choices(),
            enabled_label(policy.enabled),
        ),
        attempts = html::text_field(
            "Max attempts",
            "max_restart_attempts",
            &policy.max_restart_attempts.to_string(),
        ),
        base = html::text_field(
            "Base delay (s)",
            "base_delay_seconds",
            &policy.base_delay.as_secs().to_string()
        ),
        cap = html::text_field(
            "Max delay (s)",
            "max_delay_seconds",
            &policy.max_delay.as_secs().to_string()
        ),
    )
}

#[allow(clippy::too_many_arguments)]
fn monitor_form(
    title: &str,
    action: &str,
    describe: &str,
    enabled: bool,
    interval_seconds: u64,
    min_seconds: u64,
    max_seconds: u64,
) -> String {
    format!(
        r#"<h2>{title}</h2>
<p class="muted">{describe} — accepted range {min}s to {max}s.</p>
<form class="filters" method="post" action="{action}">
{enabled_field}
{interval}
<button type="submit">Save</button>
</form>"#,
        title = html::escape(title),
        describe = html::escape(describe),
        min = min_seconds,
        max = max_seconds,
        action = html::escape(action),
        enabled_field = html::choice_field(
            "Status",
            "enabled",
            &enabled_choices(),
            enabled_label(enabled),
        ),
        interval = html::text_field(
            "Interval (s)",
            "interval_seconds",
            &interval_seconds.to_string()
        ),
    )
}

fn upgrade_facts(policy: &RuntimeUpgradePolicy) -> String {
    let facts = [
        ("Status", enabled_label(policy.enabled).to_string()),
        ("Interval", format!("{} minutes", policy.interval_minutes)),
        (
            "Signed catalog",
            enabled_label(policy.require_signed_catalog).to_string(),
        ),
        ("Nodes per run", policy.max_nodes_per_run.to_string()),
        (
            "Maintenance window",
            if policy.maintenance_window_enabled {
                format!(
                    "{:02}:{:02}–{:02}:{:02} UTC",
                    policy.maintenance_window_start_minute_utc / 60,
                    policy.maintenance_window_start_minute_utc % 60,
                    policy.maintenance_window_end_minute_utc / 60,
                    policy.maintenance_window_end_minute_utc % 60
                )
            } else {
                "unbounded".to_string()
            },
        ),
        (
            "Catalog profile",
            policy
                .catalog_profile_id
                .clone()
                .unwrap_or_else(|| "default".to_string()),
        ),
    ];
    let rows = facts
        .iter()
        .map(|(label, value)| html::row(&[html::cell(label), html::cell(value)]))
        .collect::<Vec<_>>();
    format!(
        "{}\n{}",
        html::note("Scheduled upgrades are driven by the runtime catalog; edit this policy through the CLI or the Rust API."),
        html::table(&["Setting", "Value"], &rows)
    )
}

pub fn enabled_label(enabled: bool) -> &'static str {
    if enabled {
        "Enabled"
    } else {
        "Disabled"
    }
}

pub fn enabled_choices() -> Vec<String> {
    ENABLED_CHOICES
        .iter()
        .map(|choice| (*choice).to_string())
        .collect()
}

/// The inverse of [`enabled_label`]: anything but the explicit "Disabled"
/// choice stays on, so a hand-edited post cannot silently disable a monitor.
pub fn choice_is_enabled(raw: &str) -> bool {
    !raw.trim().eq_ignore_ascii_case("disabled")
}
