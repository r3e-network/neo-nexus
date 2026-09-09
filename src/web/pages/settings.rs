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
    web::assets::DensityMode,
};

use super::super::{html, WebState};

const ENABLED_CHOICES: &[&str] = &["Enabled", "Disabled"];
const DENSITY_CHOICES: &[&str] = &["Comfortable", "Compact"];

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
    let density = repository
        .load_app_ui_density()?
        .as_deref()
        .map_or(DensityMode::DEFAULT, DensityMode::from_str);
    Ok(format!(
        r#"<h1>Settings</h1>
{engine_note}
{appearance}
{watchdog}
{rpc_health}
{federation}
<h2>Runtime upgrades</h2>
{upgrade}"#,
        engine_note = html::notice(
            "ok",
            "Applied by this workbench process: the supervision engine reads these on its own tick, so a saved change takes effect without a restart.",
        ),
        appearance = density_form(density),
        watchdog = watchdog_form(&watchdog),
        rpc_health = monitor_form(
            "rpc-health",
            "RPC health monitor",
            "/settings/rpc-health",
            &rpc_health.describe(),
            rpc_health.enabled,
            rpc_health.interval_seconds,
            RpcHealthMonitorPolicy::MIN_INTERVAL_SECONDS,
            RpcHealthMonitorPolicy::MAX_INTERVAL_SECONDS,
        ),
        federation = monitor_form(
            "federation",
            "Federation monitor",
            "/settings/federation",
            &federation.describe(),
            federation.enabled,
            federation.interval_seconds,
            RemoteFederationMonitorPolicy::MIN_INTERVAL_SECONDS,
            RemoteFederationMonitorPolicy::MAX_INTERVAL_SECONDS,
        ),
        upgrade = upgrade_form(&upgrade),
    ))
}

fn density_form(density: DensityMode) -> String {
    format!(
        r#"<h2>Appearance</h2>
<p class="muted">Compact density tightens row height and spacing for denser fleets; comfortable is the default. Applied on the next page load.</p>
<form class="filters" method="post" action="/settings/density">
{density_field}
<button type="submit">Save</button>
</form>"#,
        density_field = html::ChoiceField {
            id: Some("appearance-density"),
            label: "UI density",
            name: "ui_density",
            options: &density_choices(),
            selected: density_label(density),
            ..Default::default()
        }
        .render(),
    )
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
        enabled = html::ChoiceField {
            id: Some("watchdog-enabled"),
            label: "Status",
            name: "enabled",
            options: &enabled_choices(),
            selected: enabled_label(policy.enabled),
            ..Default::default()
        }
        .render(),
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
    id_prefix: &str,
    title: &str,
    action: &str,
    describe: &str,
    enabled: bool,
    interval_seconds: u64,
    min_seconds: u64,
    max_seconds: u64,
) -> String {
    let enabled_id = format!("{id_prefix}-enabled");
    let interval_id = format!("{id_prefix}-interval");
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
        enabled_field = html::ChoiceField {
            id: Some(&enabled_id),
            label: "Status",
            name: "enabled",
            options: &enabled_choices(),
            selected: enabled_label(enabled),
            ..Default::default()
        }
        .render(),
        interval = html::TextField {
            id: Some(&interval_id),
            label: "Interval (s)",
            name: "interval_seconds",
            value: &interval_seconds.to_string(),
            ..Default::default()
        }
        .render(),
    )
}

/// The runtime upgrade policy the supervision engine reads on its own tick. The
/// two timestamp fields are its own record of what it did, so the form omits
/// them and shows their current values as read-only context instead.
fn upgrade_form(policy: &RuntimeUpgradePolicy) -> String {
    let last_checked = policy
        .last_checked_at_unix
        .map_or_else(|| "never".to_string(), |unix| format!("{unix} (unix)"));
    let last_applied = policy
        .last_applied_at_unix
        .map_or_else(|| "never".to_string(), |unix| format!("{unix} (unix)"));
    format!(
        r#"<p class="muted">The supervision engine reads this policy on its own tick and upgrades eligible fleet nodes within the batch size and maintenance window it defines. Last checked {last_checked}; last applied {last_applied}.</p>
<form class="filters" method="post" action="/settings/runtime-upgrade">
{enabled}
{profile}
{interval}
{signed}
{batch}
{window_enabled}
{window_start}
{window_end}
{wave_delay}
<button type="submit">Save</button>
</form>"#,
        last_checked = html::escape(&last_checked),
        last_applied = html::escape(&last_applied),
        enabled = upgrade_checkbox("upgrade-enabled", "Enabled", "enabled", policy.enabled),
        profile = html::TextField {
            id: Some("upgrade-catalog-profile"),
            label: "Catalog profile",
            name: "catalog_profile_id",
            value: policy.catalog_profile_id.as_deref().unwrap_or(""),
            help: Some("Required while enabled; leave blank to clear."),
            ..Default::default()
        }
        .render(),
        interval = upgrade_number(
            "upgrade-interval",
            "Interval (minutes)",
            "interval_minutes",
            &policy.interval_minutes.to_string(),
            RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES,
            RuntimeUpgradePolicy::MAX_INTERVAL_MINUTES,
        ),
        signed = upgrade_checkbox(
            "upgrade-signed",
            "Require signed catalog",
            "require_signed_catalog",
            policy.require_signed_catalog,
        ),
        batch = upgrade_number(
            "upgrade-batch",
            "Nodes per run",
            "max_nodes_per_run",
            &policy.max_nodes_per_run.to_string(),
            1,
            RuntimeUpgradePolicy::MAX_NODES_PER_RUN as u64,
        ),
        window_enabled = upgrade_checkbox(
            "upgrade-window",
            "Maintenance window",
            "maintenance_window_enabled",
            policy.maintenance_window_enabled,
        ),
        window_start = upgrade_number(
            "upgrade-window-start",
            "Window start (minute UTC)",
            "maintenance_window_start_minute_utc",
            &policy.maintenance_window_start_minute_utc.to_string(),
            0,
            u64::from(RuntimeUpgradePolicy::MINUTES_PER_DAY),
        ),
        window_end = upgrade_number(
            "upgrade-window-end",
            "Window end (minute UTC)",
            "maintenance_window_end_minute_utc",
            &policy.maintenance_window_end_minute_utc.to_string(),
            0,
            u64::from(RuntimeUpgradePolicy::MINUTES_PER_DAY),
        ),
        wave_delay = upgrade_number(
            "upgrade-wave-delay",
            "Wave delay (minutes)",
            "wave_delay_minutes",
            &policy.wave_delay_minutes.to_string(),
            0,
            RuntimeUpgradePolicy::MAX_WAVE_DELAY_MINUTES,
        ),
    )
}

/// A bounded number input that mirrors the field markup of [`html::TextField`],
/// so the runtime-upgrade form matches the other settings forms while still
/// carrying the `min`/`max` the domain enforces server-side.
fn upgrade_number(id: &str, label: &str, name: &str, value: &str, min: u64, max: u64) -> String {
    format!(
        r#"<label class="field" for="{id}"><span>{label}</span><input type="number" id="{id}" name="{name}" value="{value}" min="{min}" max="{max}"></label>"#,
        id = html::escape(id),
        label = html::escape(label),
        name = html::escape(name),
        value = html::escape(value),
    )
}

/// A checkbox whose submitted value is `true`, so an unchecked box sends nothing
/// and the handler reads it as off through a defaulted `Option<bool>`.
fn upgrade_checkbox(id: &str, label: &str, name: &str, checked: bool) -> String {
    let checked_attr = if checked { " checked" } else { "" };
    format!(
        r#"<label class="field" for="{id}"><span>{label}</span><input type="checkbox" id="{id}" name="{name}" value="true"{checked_attr}></label>"#,
        id = html::escape(id),
        label = html::escape(label),
        name = html::escape(name),
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

/// The capitalised label for a density mode, matching the `DENSITY_CHOICES`
/// entries so the dropdown pre-selects the stored preference.
pub fn density_label(density: DensityMode) -> &'static str {
    match density {
        DensityMode::Comfortable => "Comfortable",
        DensityMode::Compact => "Compact",
    }
}

pub fn density_choices() -> Vec<String> {
    DENSITY_CHOICES
        .iter()
        .map(|choice| (*choice).to_string())
        .collect()
}

/// The inverse of [`enabled_label`]: anything but the explicit "Disabled"
/// choice stays on, so a hand-edited post cannot silently disable a monitor.
pub fn choice_is_enabled(raw: &str) -> bool {
    !raw.trim().eq_ignore_ascii_case("disabled")
}

#[cfg(test)]
#[path = "../../../tests/unit/web/settings/tests.rs"]
mod tests;
