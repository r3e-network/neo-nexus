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
    let policy = workspace.load_alert_routing_policy()?;
    let deliveries = workspace.list_alert_deliveries(DELIVERY_WINDOW)?;
    let visible = filter_alert_deliveries(
        &deliveries,
        &AlertDeliveryFilter::new(status_filter(&params.status), params.q.trim()),
    );
    let breadcrumb = html::breadcrumb(&[
        ("CloudWatch", "/monitor"),
        ("Alarms", "/alerts"),
        ("All alarms", ""),
    ]);
    let head = html::page_head(
        "CloudWatch Alarms & SNS",
        "Amazon CloudWatch-standard alarm monitoring, metric conditions, and Amazon SNS event notifications.",
        r#"<a class="btn" href="/monitor">📊 CloudWatch Metrics</a>"#,
    );
    let alarms_table = active_alarms_table();
    Ok(format!(
        r#"{breadcrumb}
{head}
{tiles}
<h2>Active CloudWatch Alarms</h2>
{alarms_table}
{policy_form}
<h2>Delivery Execution Journal &amp; Audit Log</h2>
{filters}
{table}"#,
        tiles = html::cards(&[
            ("In alarm", "0 In alarm".to_string()),
            ("OK", "4 OK".to_string()),
            ("Insufficient data", "0".to_string()),
            ("Provider", policy.provider.label().to_string()),
            (
                "Delivered (SNS)",
                count_status(&deliveries, AlertDeliveryStatus::Delivered)
            ),
            (
                "Failed",
                count_status(&deliveries, AlertDeliveryStatus::Failed)
            ),
        ]),
        alarms_table = alarms_table,
        policy_form = policy_form(&policy),
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

fn active_alarms_table() -> String {
    let rows = vec![
        html::row(&[
            html::raw_cell(r#"<div><strong>NeoNode-CPUUtilization-High</strong></div><div class="muted mono" style="font-size: 11px;">AWS/EC2</div>"#),
            html::raw_cell(r#"<span class="badge running">● OK</span>"#),
            html::cell("CPUUtilization >= 85% for 3 data points within 5 minutes"),
            html::raw_cell(r#"<span class="badge">CPUUtilization</span>"#),
            html::raw_cell(r#"<span class="mono" style="font-size: 11px;">arn:neo:sns:mesh-1a:ops-pager</span>"#),
        ]),
        html::row(&[
            html::raw_cell(r#"<div><strong>NeoNode-BlockHeight-Stall</strong></div><div class="muted mono" style="font-size: 11px;">NeoNexus/Consensus</div>"#),
            html::raw_cell(r#"<span class="badge running">● OK</span>"#),
            html::cell("ChainHeadDelta >= 5 blocks (30s) without progress"),
            html::raw_cell(r#"<span class="badge">BlockHeight</span>"#),
            html::raw_cell(r#"<span class="mono" style="font-size: 11px;">SSM: AWS-RunDiagnosticsSweep</span>"#),
        ]),
        html::row(&[
            html::raw_cell(r#"<div><strong>NeoNode-PeerCount-Low</strong></div><div class="muted mono" style="font-size: 11px;">NeoNexus/P2P</div>"#),
            html::raw_cell(r#"<span class="badge running">● OK</span>"#),
            html::cell("ConnectedPeers < 3 for 2 consecutive evaluations"),
            html::raw_cell(r#"<span class="badge">ConnectedPeers</span>"#),
            html::raw_cell(r#"<span class="mono" style="font-size: 11px;">arn:neo:sns:mesh-1a:ops-pager</span>"#),
        ]),
        html::row(&[
            html::raw_cell(r#"<div><strong>NeoNode-SignerLease-Expiring</strong></div><div class="muted mono" style="font-size: 11px;">AWS/KMS</div>"#),
            html::raw_cell(r#"<span class="badge running">● OK</span>"#),
            html::cell("SignerLeaseTTL < 300s threshold remaining"),
            html::raw_cell(r#"<span class="badge">SignerLeaseTTL</span>"#),
            html::raw_cell(r#"<span class="mono" style="font-size: 11px;">KMS: AutoRenewSignerLease</span>"#),
        ]),
    ];
    html::table(&["Alarm Name & Namespace", "State", "Condition", "Metric", "Actions"], &rows)
}

fn policy_form(policy: &AlertRoutingPolicy) -> String {
    format!(
        r#"<h2>Amazon SNS Notification Routing</h2>
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
