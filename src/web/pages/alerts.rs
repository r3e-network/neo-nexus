//! Alerts: where the workbench sends what, and whether it arrived. The routing
//! policy edits reuse the domain's own `normalized()` and `validation_message()`,
//! and the webhook target is shown redacted — those URLs carry the provider
//! token, so the page must not echo it back.

use axum::{
    extract::{Query, RawQuery, State},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    core::operations::{
        filter_alert_deliveries, AlertDelivery, AlertDeliveryFilter, AlertDeliveryStatus,
        AlertProvider, AlertRoutingPolicy, EventSeverity,
    },
    redaction::redact_sensitive_text,
    repository::Repository,
};

use super::super::{html, WebState};

const DELIVERY_WINDOW: usize = 50;

#[derive(Default, Deserialize)]
pub struct AlertQuery {
    #[serde(default)]
    status: String,
    #[serde(default)]
    q: String,
}

pub async fn alerts(
    State(state): State<WebState>,
    RawQuery(flash): RawQuery,
    Query(params): Query<AlertQuery>,
) -> Response {
    let body = match render_body(&state.repository, &params) {
        Ok(body) => body,
        Err(error) => html::note(&format!("failed to load alerting state: {error}")),
    };
    Html(html::layout(
        "Alerts",
        "alerts",
        &html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

fn render_body(repository: &Repository, params: &AlertQuery) -> anyhow::Result<String> {
    let policy = repository.load_alert_routing_policy()?;
    let deliveries = repository.list_alert_deliveries(DELIVERY_WINDOW)?;
    let visible = filter_alert_deliveries(
        &deliveries,
        &AlertDeliveryFilter::new(status_filter(&params.status), params.q.trim()),
    );
    Ok(format!(
        r#"<h1>Alerts</h1>
{tiles}
{policy_form}
<h2>Recent deliveries</h2>
{filters}
{table}"#,
        tiles = html::cards(&[
            ("Routing", enabled_label(policy.enabled).to_string()),
            ("Provider", policy.provider.label().to_string()),
            ("At least", policy.min_severity.label().to_string()),
            (
                "Delivered",
                count_status(&deliveries, AlertDeliveryStatus::Delivered)
            ),
            (
                "Failed",
                count_status(&deliveries, AlertDeliveryStatus::Failed)
            ),
            (
                "Skipped",
                count_status(&deliveries, AlertDeliveryStatus::Skipped)
            ),
        ]),
        policy_form = policy_form(&policy),
        filters = html::filter_form("/alerts", &[("status", &params.status), ("q", &params.q)]),
        table = delivery_table(&visible),
    ))
}

fn policy_form(policy: &AlertRoutingPolicy) -> String {
    format!(
        r#"<h2>Routing policy</h2>
<p class="muted">{describe}</p>
<form class="filters" method="post" action="/alerts/routing">
{enabled}
{provider}
{severity}
{target}
{timeout}
<button type="submit">Save</button>
</form>
<p class="muted">Current target: {masked}</p>
{warning}"#,
        describe = html::escape(&policy.describe()),
        enabled = html::choice_field(
            "Status",
            "enabled",
            &enabled_choices(),
            enabled_label(policy.enabled)
        ),
        provider = html::choice_field(
            "Provider",
            "provider",
            &provider_choices(),
            policy.provider.label()
        ),
        severity = html::choice_field(
            "Minimum severity",
            "min_severity",
            &severity_choices(),
            policy.min_severity.label()
        ),
        target = r#"<label class="field"><span>Webhook target</span><input name="webhook_url" value="" placeholder="leave blank to keep"></label>"#,
        timeout = html::text_field(
            "Timeout (s)",
            "timeout_seconds",
            &policy.timeout_seconds.to_string()
        ),
        masked = html::escape(&masked_target(policy)),
        warning = policy
            .validation_message()
            .map(|message| html::note(&format!("policy needs attention: {message}")))
            .unwrap_or_default(),
    )
}

/// The stored target may hold a provider token. For Slack, Discord, and
/// Telegram the credential *is* the URL path, so the page keeps only the scheme
/// and host — enough to recognise which hook is configured — and never echoes
/// the rest back. Anything that is not a URL falls through to the generic
/// redactor; an unparseable value is never shown verbatim.
fn safe_target(value: &str) -> String {
    url::Url::parse(value)
        .ok()
        .and_then(|parsed| {
            parsed
                .host_str()
                .map(|host| format!("{}://{}…", parsed.scheme(), host))
        })
        .unwrap_or_else(|| redact_sensitive_text(value))
}

fn masked_target(policy: &AlertRoutingPolicy) -> String {
    policy
        .webhook_url
        .as_deref()
        .map(safe_target)
        .unwrap_or_else(|| "not configured".to_string())
}

fn delivery_table(deliveries: &[AlertDelivery]) -> String {
    if deliveries.is_empty() {
        return html::note("No alerts have been routed from this workspace yet.");
    }
    let rows = deliveries
        .iter()
        .map(|delivery| {
            html::row(&[
                html::cell(&delivery.attempted_at_unix.to_string()),
                html::raw_cell(&delivery_badge(delivery.status)),
                html::cell(&delivery.route_label),
                html::cell(&safe_target(&delivery.target)),
                html::cell(
                    &delivery
                        .http_status
                        .map_or_else(|| "—".to_string(), |status| status.to_string()),
                ),
                html::cell(&redact_sensitive_text(&delivery.message)),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Attempted (unix)",
            "Status",
            "Route",
            "Target",
            "HTTP",
            "Message",
        ],
        &rows,
    )
}

fn delivery_badge(status: AlertDeliveryStatus) -> String {
    let class = match status {
        AlertDeliveryStatus::Delivered => "badge running",
        AlertDeliveryStatus::Failed => "badge error",
        AlertDeliveryStatus::Skipped => "badge stopped",
    };
    format!(
        r#"<span class="{class}">{}</span>"#,
        html::escape(status.label())
    )
}

fn count_status(deliveries: &[AlertDelivery], wanted: AlertDeliveryStatus) -> String {
    deliveries
        .iter()
        .filter(|delivery| delivery.status == wanted)
        .count()
        .to_string()
}

fn status_filter(raw: &str) -> Option<AlertDeliveryStatus> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "delivered" => Some(AlertDeliveryStatus::Delivered),
        "failed" => Some(AlertDeliveryStatus::Failed),
        "skipped" => Some(AlertDeliveryStatus::Skipped),
        _ => None,
    }
}

pub fn enabled_label(enabled: bool) -> &'static str {
    if enabled {
        "Enabled"
    } else {
        "Disabled"
    }
}

fn enabled_choices() -> Vec<String> {
    ["Enabled", "Disabled"]
        .iter()
        .map(|label| label.to_string())
        .collect()
}

fn provider_choices() -> Vec<String> {
    AlertProvider::ALL
        .iter()
        .map(|provider| provider.label().to_string())
        .collect()
}

fn severity_choices() -> Vec<String> {
    EventSeverity::ALL
        .iter()
        .map(|severity| severity.label().to_string())
        .collect()
}
