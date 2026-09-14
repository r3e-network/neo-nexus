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
    core::workspace_queries::WorkspaceQueries,
    redaction::redact_sensitive_text,
};

use super::super::{html, time, WebState};

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
    let body = match render_body(&state.workspace, &params) {
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

fn render_body(workspace: &WorkspaceQueries, params: &AlertQuery) -> anyhow::Result<String> {
    let nodes = workspace.list_nodes()?;
    let policy = workspace.load_alert_routing_policy()?;
    let deliveries = workspace.list_alert_deliveries(DELIVERY_WINDOW)?;
    let visible = filter_alert_deliveries(
        &deliveries,
        &AlertDeliveryFilter::new(status_filter(&params.status), params.q.trim()),
    );
    let breadcrumb = html::breadcrumb(&[("Operations", "/operations"), ("Alerts", "")]);
    let head = html::page_head(
        "Alerts",
        "Where journal events are sent, and whether they arrived.",
        r#"<a class="btn" href="/events">Event journal</a>"#,
    );
    Ok(format!(
        r#"{breadcrumb}
{head}
{tiles}
{no_alarms}
{policy_form}
<h2>Delivery journal</h2>
{filters}
{table}"#,
        // The four tiles that stood at the head of this page read
        // "0 In alarm", "4 OK" and "0 Insufficient data" as string literals,
        // above a table of four alarms — block-height stall, peer-count low,
        // signer-lease expiring, high CPU — every one of them hardcoded to
        // "● OK". Nothing in the workspace evaluates any of those conditions,
        // and the metrics they name are emitted nowhere. An operator read that
        // page as evidence their fleet was being watched for chain liveness.
        tiles = html::cards(&[
            ("Provider", policy.provider.label().to_string()),
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
        // The warning that stood here — "NeoNexus does not evaluate alarm
        // conditions … a node that stops producing blocks will not raise an
        // alert" — was true when it was written and is not any more. Chain
        // health is evaluated every monitoring interval, and a change of state
        // journals a `node-health-changed` event whose severity follows the
        // state: Critical for unreachable and stalled, Warning for isolated and
        // degraded. Routing that kind is how a stall reaches a pager.
        no_alarms = html::notice(
            "info",
            "Chain health is evaluated for every node on the monitoring interval, and a change \
             of state is journalled as node-health-changed — Critical when a node becomes \
             unreachable or stalls, Warning when it becomes isolated or degraded. Route that \
             kind to page on a node that stops producing blocks. Nothing is evaluated on a \
             schedule of its own beyond that: these are journal events, not independent alarms.",
        ),
        policy_form = policy_form(&policy, &nodes),
        filters = html::typed_filter_form(
            "/alerts",
            &[],
            &[
                html::FilterControl::Select {
                    label: "Delivery status",
                    name: "status",
                    selected: &params.status,
                    options: &[
                        ("", "All deliveries"),
                        ("delivered", "Delivered"),
                        ("failed", "Failed"),
                        ("skipped", "Skipped"),
                    ],
                },
                html::FilterControl::Search {
                    label: "Search",
                    name: "q",
                    value: &params.q,
                    placeholder: "Route, target, or message",
                },
            ],
        ),
        table = delivery_table(&visible),
    ))
}

fn policy_form(policy: &AlertRoutingPolicy, nodes: &[crate::types::NodeConfig]) -> String {
    format!(
        r#"<h2>Where events are sent</h2>
<p class="muted">{describe}</p>
<form method="post" action="/alerts/routing">
<div class="filters">
{enabled}
{provider}
{severity}
{target}
{timeout}
</div>
<div class="grid" style="grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 14px; margin: 12px 0;">
  <label class="field"><span>Only these kinds</span>{kinds}<small class="muted">Nothing selected means every kind. Hold ⌘ or Ctrl to pick several.</small></label>
  <label class="field"><span>Only these nodes</span>{node_scope}<small class="muted">Nothing selected means any node. Workspace-wide events are excluded once you name nodes.</small></label>
</div>
<button type="submit">Save</button>
</form>
<form method="post" action="/alerts/routing/preview" style="margin-top: -6px;">
  <button type="submit" class="btn small" title="Build the exact request this provider would receive, with credentials redacted. Sends nothing.">Show what would be sent</button>
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
        kinds = multi_select("kinds", &kind_options(), &selected_kinds(policy)),
        node_scope = multi_select("node_ids", &node_options(nodes), &policy.node_ids),
        masked = html::escape(&masked_target(policy)),
        warning = policy
            .validation_message()
            .map(|message| html::note(&format!("policy needs attention: {message}")))
            .unwrap_or_default(),
    )
}

/// A multi-select. Nothing selected posts no field at all, which the handler
/// reads as "no scope" — that is, everything.
fn multi_select(name: &str, options: &[(String, String)], selected: &[String]) -> String {
    let rendered = options
        .iter()
        .map(|(value, label)| {
            let is_selected = if selected.iter().any(|chosen| chosen == value) {
                " selected"
            } else {
                ""
            };
            format!(
                r#"<option value="{value}"{is_selected}>{label}</option>"#,
                value = html::escape(value),
                label = html::escape(label),
            )
        })
        .collect::<String>();
    format!(r#"<select name="{name}" multiple size="6">{rendered}</select>"#)
}

/// The kinds worth offering as a routing scope.
///
/// Every one of the 93 would be a wall of names, most of which no code path
/// constructs. These are the ones that describe something happening *to a
/// node* — which is what an operator routes on.
fn kind_options() -> Vec<(String, String)> {
    use crate::events::EventKind;
    [
        EventKind::NodeHealthChanged,
        EventKind::NodeExited,
        EventKind::NodeStartFailed,
        EventKind::NodeStarted,
        EventKind::NodeStopped,
        EventKind::NodeRestarted,
        EventKind::WatchdogRestarted,
        EventKind::WatchdogExhausted,
        EventKind::RpcHealthChecked,
        EventKind::RemoteServerProbed,
        EventKind::RuntimeApplied,
        EventKind::NodeSignerBound,
    ]
    .into_iter()
    .map(|kind| (kind.label().to_string(), kind.label().to_string()))
    .collect()
}

fn selected_kinds(policy: &AlertRoutingPolicy) -> Vec<String> {
    policy
        .kinds
        .iter()
        .map(|kind| kind.label().to_string())
        .collect()
}

fn node_options(nodes: &[crate::types::NodeConfig]) -> Vec<(String, String)> {
    nodes
        .iter()
        .map(|node| (node.id.clone(), node.name.clone()))
        .collect()
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
                html::raw_cell(&time::time_cell(Some(delivery.attempted_at_unix))),
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
        &["Attempted", "Status", "Route", "Target", "HTTP", "Message"],
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
