//! Settings and policy form save handlers.

use std::{str::FromStr, time::Duration};

use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    core::{
        operations::{
            AlertProvider, AlertRoutingPolicy, EventKind, EventSeverity, NewRuntimeEvent,
            RemoteFederationMonitorPolicy, RpcHealthMonitorPolicy,
        },
        runtime::{validate_runtime_upgrade_policy, RestartPolicy, RuntimeUpgradePolicy},
        workspace_queries::WorkspaceQueries,
    },
    web::form_state::{FieldErrors, GENERAL},
};

use super::super::{assets::DensityMode, html, pages::settings, WebState};
use settings::{MonitorDraft, RuntimeUpgradeDraft, WatchdogDraft};

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

/// The runtime upgrade policy form as the operator typed it.
///
/// The form used to be submitted, judged, and thrown away: a value the domain
/// refused sent the operator back to a page that showed the *saved* policy
/// and one sentence saying why the save failed. Everything they had typed —
/// including the parts that were fine — was gone.
///
/// This keeps the posted text so the form can be rendered back with it. The
/// values stay text because the point is to show what was typed, including
/// the value that was wrong.
impl RuntimeUpgradeDraft {
    fn from_form(form: &RuntimeUpgradeForm) -> Self {
        Self {
            enabled: form.enabled.unwrap_or(false),
            catalog_profile_id: form.catalog_profile_id.clone(),
            interval_minutes: form.interval_minutes.clone(),
            require_signed_catalog: form.require_signed_catalog.unwrap_or(false),
            max_nodes_per_run: form.max_nodes_per_run.clone(),
            maintenance_window_enabled: form.maintenance_window_enabled.unwrap_or(false),
            maintenance_window_start_minute_utc: form.maintenance_window_start_minute_utc.clone(),
            maintenance_window_end_minute_utc: form.maintenance_window_end_minute_utc.clone(),
            wave_delay_minutes: form.wave_delay_minutes.clone(),
        }
    }
}

impl WatchdogDraft {
    fn from_form(form: &WatchdogForm) -> Self {
        Self {
            enabled: settings::choice_is_enabled(&form.enabled),
            max_restart_attempts: form.max_restart_attempts.clone(),
            base_delay_seconds: form.base_delay_seconds.clone(),
            max_delay_seconds: form.max_delay_seconds.clone(),
            jitter_enabled: form
                .jitter_enabled
                .as_deref()
                .is_some_and(|value| value.eq_ignore_ascii_case("true") || value == "on"),
        }
    }
}

impl MonitorDraft {
    fn from_form(form: &MonitorForm) -> Self {
        Self {
            enabled: settings::choice_is_enabled(&form.enabled),
            interval_seconds: form.interval_seconds.clone(),
        }
    }
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
    /// The kinds of event worth waking someone for. A multi-select posts one
    /// field per selection, so this is a list; empty means every kind.
    #[serde(default)]
    kinds: Vec<String>,
    /// Which nodes. Empty means any.
    #[serde(default)]
    node_ids: Vec<String>,
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
///
/// A refused save comes back as the settings page with the operator's own input
/// still in it: four numeric fields are not worth retyping to correct one.
pub async fn save_watchdog(
    State(state): State<WebState>,
    Form(input): Form<WatchdogForm>,
) -> Response {
    let draft = WatchdogDraft::from_form(&input);
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
    finish(&state, outcome, watchdog_field_for, |workspace, errors| {
        settings::render_with_watchdog_draft(workspace, &draft, errors)
    })
}

/// Save RPC health monitor policy.
pub async fn save_rpc_health_monitor(
    State(state): State<WebState>,
    Form(input): Form<MonitorForm>,
) -> Response {
    let draft = MonitorDraft::from_form(&input);
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
    finish(&state, outcome, monitor_field_for, |workspace, errors| {
        settings::render_with_rpc_health_draft(workspace, &draft, errors)
    })
}

/// Save remote federation monitor policy.
pub async fn save_federation_monitor(
    State(state): State<WebState>,
    Form(input): Form<MonitorForm>,
) -> Response {
    let draft = MonitorDraft::from_form(&input);
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
    finish(&state, outcome, monitor_field_for, |workspace, errors| {
        settings::render_with_federation_draft(workspace, &draft, errors)
    })
}

/// Save runtime upgrade policy.
///
/// A refused save comes back as the settings page with the operator's own
/// input in it and the error beside the field that caused it. It used to
/// redirect with one sentence and the whole form gone.
pub async fn save_runtime_upgrade_policy(
    State(state): State<WebState>,
    Form(input): Form<RuntimeUpgradeForm>,
) -> Response {
    let draft = RuntimeUpgradeDraft::from_form(&input);
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
    finish(&state, outcome, upgrade_field_for, |workspace, errors| {
        settings::render_with_upgrade_draft(workspace, &draft, errors)
    })
}

/// The tail every settings form shares.
///
/// A save that worked redirects: the page it lands on re-reads the policy from
/// the workspace, so a 200 here would claim a state this handler has no reason
/// to draw from memory. A save that was refused comes back on the page, with the
/// operator's text still in the form and the message beside the field it belongs
/// to — and if that page cannot be read back either, the redirect says both what
/// was wrong and that the form could not be kept.
fn finish(
    state: &WebState,
    outcome: anyhow::Result<String>,
    blame: fn(&anyhow::Error) -> &'static str,
    draw: impl FnOnce(&WorkspaceQueries, &FieldErrors) -> anyhow::Result<String>,
) -> Response {
    match outcome {
        Ok(message) => {
            let flash = html::Flash::ok(message);
            Redirect::to(&flash.redirect("/settings")).into_response()
        }
        Err(error) => {
            // The domain's message says what is wrong in its own words; the
            // mapper says where. Together they turn "not saved: …" into an
            // error under the control the operator has to change.
            let mut errors = FieldErrors::new();
            errors.insert(blame(&error), error.to_string());
            match draw(&state.workspace, &errors) {
                Ok(body) => Html(html::layout("Settings", "settings", "", &body)).into_response(),
                Err(render_error) => {
                    let flash = html::Flash::danger(format!(
                        "not saved: {error} (the page could not be redrawn: {render_error})"
                    ));
                    Redirect::to(&flash.redirect("/settings")).into_response()
                }
            }
        }
    }
}

/// Which watchdog control a domain message is about.
///
/// The domain validates the restart budget as a whole, so its sentence is the
/// only thing that says what was wrong. Mapping it back to a field is what turns
/// that sentence into a message beside the box to retype.
fn watchdog_field_for(error: &anyhow::Error) -> &'static str {
    let message = error.to_string();
    if message.contains("max attempts") {
        "max_restart_attempts"
    } else if message.contains("base delay") {
        "base_delay_seconds"
    } else if message.contains("max delay") {
        "max_delay_seconds"
    } else if message.contains("jitter") {
        "jitter_enabled"
    } else {
        GENERAL
    }
}

/// Which monitor control a domain message is about.
///
/// The two monitors share this form: one interval, one switch, and nothing else
/// that can fail, so the interval is the only field a refusal can belong to.
fn monitor_field_for(error: &anyhow::Error) -> &'static str {
    if error.to_string().contains("interval") {
        "interval_seconds"
    } else {
        GENERAL
    }
}

/// Which field of the upgrade form a domain message is about.
///
/// The domain validates the policy as a whole, so its message is the only
/// thing that says what was wrong. Mapping it back to a field is what turns
/// "not saved: …" into an error under the control that caused it.
fn upgrade_field_for(error: &anyhow::Error) -> &'static str {
    let message = error.to_string();
    if message.contains("catalog profile") {
        "catalog_profile_id"
    } else if message.contains("interval") {
        "interval_minutes"
    } else if message.contains("batch size") {
        "max_nodes_per_run"
    } else if message.contains("maintenance window minutes") {
        "maintenance_window_start_minute_utc"
    } else if message.contains("maintenance window") {
        "maintenance_window_end_minute_utc"
    } else if message.contains("wave delay") {
        "wave_delay_minutes"
    } else {
        GENERAL
    }
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
            // An unparseable kind is an error rather than a silent drop: a
            // route that quietly narrows to fewer kinds than the operator
            // selected is a rule that stops covering what it was written for.
            kinds: input
                .kinds
                .iter()
                .map(|raw| raw.trim())
                .filter(|raw| !raw.is_empty())
                .map(|raw| {
                    EventKind::from_str(raw)
                        .map_err(|_| anyhow::anyhow!("{raw} is not an event kind"))
                })
                .collect::<anyhow::Result<Vec<_>>>()?,
            node_ids: input
                .node_ids
                .iter()
                .map(|raw| raw.trim().to_string())
                .filter(|raw| !raw.is_empty())
                .collect(),
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
    let flash = match outcome {
        Ok(message) => html::Flash::ok(message),
        Err(error) => html::Flash::danger(format!("not saved: {error}")),
    };
    Redirect::to(&flash.redirect(path)).into_response()
}

/// Show what would actually be sent, without sending it.
///
/// `preview_alert_route` renders the exact provider payload and header set, with
/// credentials redacted, and was reachable only from `--alert-preview`. So an
/// operator who pasted a webhook URL into the console could not tell whether it
/// was the right shape for the provider they picked until a real incident
/// delivered — or failed to deliver — against it.
///
/// This sends nothing. It builds the request and renders it, so a mistyped
/// Slack hook is caught at configuration time rather than at 03:00.
pub async fn preview_alert_routing(State(state): State<WebState>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let policy = state.workspace.load_alert_routing_policy()?;
        let Some(url) = policy.webhook_url.as_deref().filter(|url| !url.is_empty()) else {
            anyhow::bail!("no webhook target is configured, so there is nothing to preview");
        };
        let sample = crate::events::RuntimeEvent {
            id: 0,
            occurred_at_unix: crate::web::time::now_unix(),
            node_id: None,
            node_name: Some("example-node".to_string()),
            kind: crate::events::EventKind::NodeHealthChanged,
            severity: policy.min_severity,
            message: "Healthy → Stalled: height 8421 has not advanced in 600s".to_string(),
        };
        let report = crate::alerts::preview_alert_route(
            policy.provider,
            url,
            &sample,
            env!("CARGO_PKG_VERSION"),
        )?;
        Ok(format!(
            "preview built for {} → {} ({} headers, {} bytes of payload); nothing was sent",
            report.provider,
            report.target,
            report.header_count,
            report.payload_json.len(),
        ))
    })();
    respond_to("/alerts", outcome)
}
