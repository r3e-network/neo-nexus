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
        workspace_queries::WorkspaceQueries,
    },
    web::{assets::DensityMode, form_state::FieldErrors},
};

use super::super::{html, WebState};

const ENABLED_CHOICES: &[&str] = &["Enabled", "Disabled"];
const DENSITY_CHOICES: &[&str] = &["Comfortable", "Compact"];

/// The runtime upgrade policy form as the operator typed it.
///
/// The values stay text because the point is to show what was typed,
/// including the value that was wrong.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct RuntimeUpgradeDraft {
    pub enabled: bool,
    pub catalog_profile_id: String,
    pub interval_minutes: String,
    pub require_signed_catalog: bool,
    pub max_nodes_per_run: String,
    pub maintenance_window_enabled: bool,
    pub maintenance_window_start_minute_utc: String,
    pub maintenance_window_end_minute_utc: String,
    pub wave_delay_minutes: String,
}

/// The watchdog restart budget as the operator typed it.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct WatchdogDraft {
    pub enabled: bool,
    pub max_restart_attempts: String,
    pub base_delay_seconds: String,
    pub max_delay_seconds: String,
    pub jitter_enabled: bool,
}

/// One of the two interval-only monitors as the operator typed it.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct MonitorDraft {
    pub enabled: bool,
    pub interval_seconds: String,
}

/// Which settings form was refused, carrying the text that was posted to it.
///
/// Exactly one can be refused per request — the four forms post to four routes
/// — so the page is drawn from one draft and three stored policies. Carrying
/// *which* panel it was, rather than a shared error map, is what stops an
/// interval error from appearing beside both interval controls: the two
/// monitors post identical `enabled` and `interval_seconds` names.
pub enum RefusedForm<'a> {
    Watchdog(&'a WatchdogDraft),
    RpcHealth(&'a MonitorDraft),
    Federation(&'a MonitorDraft),
    Upgrade(&'a RuntimeUpgradeDraft),
}

/// One empty error set, made once.
///
/// The common case is a page with no errors at all, and it is drawn on every
/// settings load.
fn no_errors() -> &'static FieldErrors {
    static EMPTY: std::sync::OnceLock<FieldErrors> = std::sync::OnceLock::new();
    EMPTY.get_or_init(FieldErrors::new)
}

/// A refused form has to look refused.
///
/// The line above each form describes the policy the supervision engine is
/// still running; the form below holds what it would not take. Both are true and
/// they disagree, and an operator who is not told so concludes the save
/// half-worked.
fn unsaved_note() -> String {
    html::note(
        "Nothing was saved: the description above is still the policy in force, and the form \
         holds what you typed and what it would not accept.",
    )
}

pub async fn settings(State(state): State<WebState>, RawQuery(query): RawQuery) -> Response {
    let body = match render_body(&state.workspace, None, no_errors()) {
        Ok(body) => body,
        Err(error) => html::note(&format!("failed to load settings: {error}")),
    };
    Html(html::layout(
        "Settings",
        "settings",
        html::flash(query.as_deref()),
        &body,
    ))
    .into_response()
}

/// The settings page as it is rendered after a refused upgrade save.
///
/// The page comes back rather than redirecting, so the operator's input is
/// still on screen and the error is beside the field that caused it.
pub fn render_with_upgrade_draft(
    workspace: &WorkspaceQueries,
    draft: &RuntimeUpgradeDraft,
    errors: &FieldErrors,
) -> anyhow::Result<String> {
    render_body(workspace, Some(RefusedForm::Upgrade(draft)), errors)
}

/// The settings page as it is rendered after a refused watchdog save.
pub fn render_with_watchdog_draft(
    workspace: &WorkspaceQueries,
    draft: &WatchdogDraft,
    errors: &FieldErrors,
) -> anyhow::Result<String> {
    render_body(workspace, Some(RefusedForm::Watchdog(draft)), errors)
}

/// The settings page as it is rendered after a refused RPC health save.
pub fn render_with_rpc_health_draft(
    workspace: &WorkspaceQueries,
    draft: &MonitorDraft,
    errors: &FieldErrors,
) -> anyhow::Result<String> {
    render_body(workspace, Some(RefusedForm::RpcHealth(draft)), errors)
}

/// The settings page as it is rendered after a refused federation save.
pub fn render_with_federation_draft(
    workspace: &WorkspaceQueries,
    draft: &MonitorDraft,
    errors: &FieldErrors,
) -> anyhow::Result<String> {
    render_body(workspace, Some(RefusedForm::Federation(draft)), errors)
}

fn render_body(
    workspace: &WorkspaceQueries,
    refused: Option<RefusedForm<'_>>,
    errors: &FieldErrors,
) -> anyhow::Result<String> {
    let watchdog = workspace.load_watchdog_policy()?;
    let rpc_health = workspace.load_rpc_health_monitor_policy()?;
    let federation = workspace.load_remote_federation_monitor_policy()?;
    let upgrade = workspace.load_runtime_upgrade_policy()?;
    let density = workspace
        .load_app_ui_density()?
        .as_deref()
        .map_or(DensityMode::DEFAULT, DensityMode::from_str);
    // Three of the four panels draw from the stored policy on any given render;
    // the fourth — if there is one — draws from what the operator typed and
    // carries the messages. Splitting them here is what keeps the two monitors
    // from sharing an error they both happen to name.
    let watchdog_refused = match &refused {
        Some(RefusedForm::Watchdog(draft)) => Some((*draft, errors)),
        _ => None,
    };
    let rpc_refused = match &refused {
        Some(RefusedForm::RpcHealth(draft)) => Some((*draft, errors)),
        _ => None,
    };
    let federation_refused = match &refused {
        Some(RefusedForm::Federation(draft)) => Some((*draft, errors)),
        _ => None,
    };
    let upgrade_refused = match &refused {
        Some(RefusedForm::Upgrade(draft)) => Some((*draft, errors)),
        _ => None,
    };
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
        watchdog = watchdog_form(&watchdog, watchdog_refused),
        rpc_health = monitor_form(
            "rpc-health",
            "RPC health monitor",
            "/settings/rpc-health",
            &rpc_health.describe(),
            rpc_health.enabled,
            rpc_health.interval_seconds,
            RpcHealthMonitorPolicy::MIN_INTERVAL_SECONDS,
            RpcHealthMonitorPolicy::MAX_INTERVAL_SECONDS,
            rpc_refused,
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
            federation_refused,
        ),
        upgrade = upgrade_form(&upgrade, upgrade_refused),
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

fn watchdog_form(
    policy: &RestartPolicy,
    refused: Option<(&WatchdogDraft, &FieldErrors)>,
) -> String {
    let errors: &FieldErrors = match refused {
        Some((_, errors)) => errors,
        None => no_errors(),
    };
    let draft = refused.map(|(draft, _)| draft);
    // The draft wins over the stored policy: it is what the operator typed.
    let (enabled, attempts, base, cap, jitter) = match draft {
        Some(draft) => (
            draft.enabled,
            draft.max_restart_attempts.clone(),
            draft.base_delay_seconds.clone(),
            draft.max_delay_seconds.clone(),
            draft.jitter_enabled,
        ),
        None => (
            policy.enabled,
            policy.max_restart_attempts.to_string(),
            policy.base_delay.as_secs().to_string(),
            policy.max_delay.as_secs().to_string(),
            policy.jitter_enabled,
        ),
    };
    let summary = refused_summary(draft.is_some(), errors);
    format!(
        r#"<h2>Watchdog restarts</h2>
<p class="muted">{describe}</p>
{summary}
<form class="filters" method="post" action="/settings/watchdog">
{enabled}
{attempts}
{base}
{cap}
{jitter}
<button type="submit">Save</button>
</form>"#,
        describe = html::escape(&policy.describe()),
        summary = summary,
        enabled = html::ChoiceField {
            id: Some("watchdog-enabled"),
            label: "Status",
            name: "enabled",
            options: &enabled_choices(),
            selected: enabled_label(enabled),
            error: crate::web::form_state::error_for(errors, "enabled"),
            ..Default::default()
        }
        .render(),
        attempts = settings_field(
            "Max attempts",
            "max_restart_attempts",
            &attempts,
            crate::web::form_state::error_for(errors, "max_restart_attempts"),
        ),
        base = settings_field(
            "Base delay (s)",
            "base_delay_seconds",
            &base,
            crate::web::form_state::error_for(errors, "base_delay_seconds"),
        ),
        cap = settings_field(
            "Max delay (s)",
            "max_delay_seconds",
            &cap,
            crate::web::form_state::error_for(errors, "max_delay_seconds"),
        ),
        jitter = checkbox_field(
            "watchdog-jitter",
            "Enable randomized jitter during restart delays",
            "jitter_enabled",
            jitter,
            crate::web::form_state::error_for(errors, "jitter_enabled"),
        ),
    )
}

/// The banner a form carries when its save was refused, or nothing at all.
///
/// Both halves come from `form_state`: how many fields to look at, and the
/// domain's own sentence when the failure belongs to none of them.
fn refused_summary(refused: bool, errors: &FieldErrors) -> String {
    if refused {
        format!(
            "{}{}",
            crate::web::form_state::banner(errors),
            unsaved_note()
        )
    } else {
        String::new()
    }
}

/// A settings text field with the message that belongs beside it.
fn settings_field(label: &str, name: &str, value: &str, error: Option<&str>) -> String {
    html::TextField {
        label,
        name,
        value,
        error,
        ..Default::default()
    }
    .render()
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
    refused: Option<(&MonitorDraft, &FieldErrors)>,
) -> String {
    let errors: &FieldErrors = match refused {
        Some((_, errors)) => errors,
        None => no_errors(),
    };
    let draft = refused.map(|(draft, _)| draft);
    let (enabled, interval_seconds) = match draft {
        Some(draft) => (draft.enabled, draft.interval_seconds.clone()),
        None => (enabled, interval_seconds.to_string()),
    };
    let summary = refused_summary(draft.is_some(), errors);
    let enabled_id = format!("{id_prefix}-enabled");
    let interval_id = format!("{id_prefix}-interval");
    format!(
        r#"<h2>{title}</h2>
<p class="muted">{describe} — accepted range {min}s to {max}s.</p>
{summary}
<form class="filters" method="post" action="{action}">
{enabled_field}
{interval}
<button type="submit">Save</button>
</form>"#,
        title = html::escape(title),
        describe = html::escape(describe),
        min = min_seconds,
        max = max_seconds,
        summary = summary,
        action = html::escape(action),
        enabled_field = html::ChoiceField {
            id: Some(&enabled_id),
            label: "Status",
            name: "enabled",
            options: &enabled_choices(),
            selected: enabled_label(enabled),
            error: crate::web::form_state::error_for(errors, "enabled"),
            ..Default::default()
        }
        .render(),
        interval = html::TextField {
            id: Some(&interval_id),
            label: "Interval (s)",
            name: "interval_seconds",
            value: &interval_seconds,
            error: crate::web::form_state::error_for(errors, "interval_seconds"),
            ..Default::default()
        }
        .render(),
    )
}

/// The runtime upgrade policy the supervision engine reads on its own tick. The
/// two timestamp fields are its own record of what it did, so the form omits
/// them and shows their current values as read-only context instead.
///
/// When a save was refused, the draft carries what the operator typed and the
/// errors say which field was wrong. The form then renders the draft rather
/// than the stored policy — the point of coming back is that the operator
/// does not have to retype the eight fields that were fine.
fn upgrade_form(
    policy: &RuntimeUpgradePolicy,
    refused: Option<(&RuntimeUpgradeDraft, &FieldErrors)>,
) -> String {
    let last_checked = policy
        .last_checked_at_unix
        .map_or_else(|| "never".to_string(), |unix| format!("{unix} (unix)"));
    let last_applied = policy
        .last_applied_at_unix
        .map_or_else(|| "never".to_string(), |unix| format!("{unix} (unix)"));
    let errors: &FieldErrors = match refused {
        Some((_, errors)) => errors,
        None => no_errors(),
    };
    let draft = refused.map(|(draft, _)| draft);
    // The draft wins over the stored policy: it is what the operator typed.
    let (enabled, catalog_profile_id, interval_minutes) = match draft {
        Some(draft) => (
            draft.enabled,
            draft.catalog_profile_id.clone(),
            draft.interval_minutes.clone(),
        ),
        None => (
            policy.enabled,
            policy.catalog_profile_id.clone().unwrap_or_default(),
            policy.interval_minutes.to_string(),
        ),
    };
    let (require_signed, max_nodes, window_enabled) = match draft {
        Some(draft) => (
            draft.require_signed_catalog,
            draft.max_nodes_per_run.clone(),
            draft.maintenance_window_enabled,
        ),
        None => (
            policy.require_signed_catalog,
            policy.max_nodes_per_run.to_string(),
            policy.maintenance_window_enabled,
        ),
    };
    let (window_start, window_end, wave_delay) = match draft {
        Some(draft) => (
            draft.maintenance_window_start_minute_utc.clone(),
            draft.maintenance_window_end_minute_utc.clone(),
            draft.wave_delay_minutes.clone(),
        ),
        None => (
            policy.maintenance_window_start_minute_utc.to_string(),
            policy.maintenance_window_end_minute_utc.to_string(),
            policy.wave_delay_minutes.to_string(),
        ),
    };
    let summary = refused_summary(draft.is_some(), errors);
    format!(
        r#"<p class="muted">The supervision engine reads this policy on its own tick and upgrades eligible fleet nodes within the batch size and maintenance window it defines. Last checked {last_checked}; last applied {last_applied}.</p>
{summary}
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
        summary = summary,
        last_checked = html::escape(&last_checked),
        last_applied = html::escape(&last_applied),
        enabled = checkbox_field(
            "upgrade-enabled",
            "Enabled",
            "enabled",
            enabled,
            crate::web::form_state::error_for(errors, "enabled"),
        ),
        profile = html::TextField {
            id: Some("upgrade-catalog-profile"),
            label: "Catalog profile",
            name: "catalog_profile_id",
            value: &catalog_profile_id,
            help: Some("Required while enabled; leave blank to clear."),
            error: crate::web::form_state::error_for(errors, "catalog_profile_id"),
            ..Default::default()
        }
        .render(),
        interval = upgrade_number(
            "upgrade-interval",
            "Interval (minutes)",
            "interval_minutes",
            &interval_minutes,
            RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES,
            RuntimeUpgradePolicy::MAX_INTERVAL_MINUTES,
            crate::web::form_state::error_for(errors, "interval_minutes"),
        ),
        // Named for what it does. This build verifies a release's SHA-256 and
        // cannot verify a publisher signature — the catalogue schema carries
        // none — so requiring one means no automatic upgrade runs at all. The
        // setting had no reader in the upgrade engine whatsoever before; its
        // only consumer was a display string.
        signed = checkbox_field(
            "upgrade-signed",
            "Require a signed catalogue (stops automatic upgrades: signatures are not verifiable in this build)",
            "require_signed_catalog",
            require_signed,
            crate::web::form_state::error_for(errors, "require_signed_catalog"),
        ),
        batch = upgrade_number(
            "upgrade-batch",
            "Nodes per run",
            "max_nodes_per_run",
            &max_nodes,
            1,
            RuntimeUpgradePolicy::MAX_NODES_PER_RUN as u64,
            crate::web::form_state::error_for(errors, "max_nodes_per_run"),
        ),
        window_enabled = checkbox_field(
            "upgrade-window",
            "Maintenance window",
            "maintenance_window_enabled",
            window_enabled,
            crate::web::form_state::error_for(errors, "maintenance_window_enabled"),
        ),
        window_start = upgrade_number(
            "upgrade-window-start",
            "Window start (minute UTC)",
            "maintenance_window_start_minute_utc",
            &window_start,
            0,
            // The domain rejects a minute of 1440 — a day has minutes 0..=1439
            // — so the browser bound has to stop one short of the day's length.
            u64::from(RuntimeUpgradePolicy::MINUTES_PER_DAY) - 1,
            crate::web::form_state::error_for(errors, "maintenance_window_start_minute_utc"),
        ),
        window_end = upgrade_number(
            "upgrade-window-end",
            "Window end (minute UTC)",
            "maintenance_window_end_minute_utc",
            &window_end,
            0,
            u64::from(RuntimeUpgradePolicy::MINUTES_PER_DAY) - 1,
            crate::web::form_state::error_for(errors, "maintenance_window_end_minute_utc"),
        ),
        wave_delay = upgrade_number(
            "upgrade-wave-delay",
            "Wave delay (minutes)",
            "wave_delay_minutes",
            &wave_delay,
            0,
            RuntimeUpgradePolicy::MAX_WAVE_DELAY_MINUTES,
            crate::web::form_state::error_for(errors, "wave_delay_minutes"),
        ),
    )
}

/// A bounded number input that mirrors the field markup of [`html::TextField`],
/// so the runtime-upgrade form matches the other settings forms while still
/// carrying the `min`/`max` the domain enforces server-side.
fn upgrade_number(
    id: &str,
    label: &str,
    name: &str,
    value: &str,
    min: u64,
    max: u64,
    error: Option<&str>,
) -> String {
    html::TextField {
        id: Some(id),
        label,
        name,
        value,
        min: Some(&min.to_string()),
        max: Some(&max.to_string()),
        step: Some("1"),
        input_type: html::TextInputType::Number,
        error,
        ..html::TextField::default()
    }
    .render()
}

/// A checkbox whose submitted value is `true`, so an unchecked box sends nothing
/// and the handler reads it as off through a defaulted `Option<bool>`.
///
/// One helper for all four: the watchdog's jitter switch and the upgrade form's
/// three switches differ only in whether a message belongs beside them.
fn checkbox_field(id: &str, label: &str, name: &str, checked: bool, error: Option<&str>) -> String {
    let id = html::escape(id);
    let checked_attr = if checked { " checked" } else { "" };
    let invalid = if error.is_some() {
        r#" aria-invalid="true""#
    } else {
        ""
    };
    // The message is part of the control, not a loose paragraph beside it: a
    // screen reader has to read the reason when it reaches the box, and the id
    // is what makes that link explicit. The same reason travels in the page's
    // summary, which is announced first.
    let messages = error.map_or_else(String::new, |message| {
        format!(
            r#"<span class="error" id="{id}-error" role="alert">{}</span>"#,
            html::escape(message)
        )
    });
    let described = if error.is_some() {
        format!(r#" aria-describedby="{id}-error""#)
    } else {
        String::new()
    };
    format!(
        r#"<label class="field{invalid_class}" for="{id}"><span>{label}</span><input type="checkbox" id="{id}" name="{name}"{checked_attr} value="true"{invalid}{described}>{messages}</label>"#,
        invalid_class = if error.is_some() { " invalid" } else { "" },
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
