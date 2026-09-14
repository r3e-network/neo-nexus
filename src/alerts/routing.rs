use crate::events::{EventSeverity, RuntimeEvent};

use super::{
    payloads::alert_delivery_request, targets::alert_target_label, text::truncate_for_message,
    AlertDeliveryReport, AlertDeliveryStatus, AlertRoutingPolicy,
};

/// Whether this event is one the operator asked to be told about.
///
/// The severity floor was the whole of this decision — `event.node_id` and
/// `event.kind` were never read — so one webhook received every event above a
/// global threshold and nothing could be scoped to anything.
///
/// An empty scope means "all", never "none". A policy that narrowed to nothing
/// by default would be an alert route that silently delivers nothing, which is
/// the failure mode alerting exists to avoid.
pub fn should_route_alert(policy: &AlertRoutingPolicy, event: &RuntimeEvent) -> bool {
    policy.enabled
        && policy
            .webhook_url
            .as_deref()
            .is_some_and(|url| !url.is_empty())
        && severity_rank(event.severity) >= severity_rank(policy.min_severity)
        && (policy.kinds.is_empty() || policy.kinds.contains(&event.kind))
        && matches_node_scope(policy, event)
}

/// Whether the event falls inside the policy's node scope.
///
/// A workspace-wide event has no `node_id`. When a policy names specific nodes,
/// such an event is **not** in scope: an operator who narrowed a route to their
/// validator did not thereby ask to hear about backup exports.
fn matches_node_scope(policy: &AlertRoutingPolicy, event: &RuntimeEvent) -> bool {
    if policy.node_ids.is_empty() {
        return true;
    }
    event
        .node_id
        .as_deref()
        .is_some_and(|node_id| policy.node_ids.iter().any(|scoped| scoped == node_id))
}

pub fn deliver_webhook_alert(
    policy: &AlertRoutingPolicy,
    event: &RuntimeEvent,
    application_version: &str,
) -> AlertDeliveryReport {
    let Some(url) = policy.webhook_url.as_deref() else {
        return skipped_delivery(event.id, "Alert routing skipped: no webhook URL");
    };
    let target = alert_target_label(url);
    let request = match alert_delivery_request(policy.provider, event, application_version, url) {
        Ok(request) => request,
        Err(error) => {
            return AlertDeliveryReport {
                event_id: event.id,
                route_label: policy.provider.to_string(),
                target,
                status: AlertDeliveryStatus::Failed,
                http_status: None,
                message: format!("invalid alert target: {error}"),
            };
        }
    };
    let payload_text = match serde_json::to_string(&request.payload) {
        Ok(payload) => payload,
        Err(error) => {
            return AlertDeliveryReport {
                event_id: event.id,
                route_label: policy.provider.to_string(),
                target,
                status: AlertDeliveryStatus::Failed,
                http_status: None,
                message: format!("failed to serialize alert payload: {error}"),
            };
        }
    };

    // Webhook targets are security boundaries: never let a provider response
    // retarget the POST (and its provider credentials) to a second URL.
    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let mut webhook_request = agent
        .post(&request.endpoint_url)
        .timeout(policy.timeout_duration())
        .set("Content-Type", "application/json")
        .set(
            "User-Agent",
            concat!("NeoNexus/", env!("CARGO_PKG_VERSION")),
        );
    for (name, value) in &request.headers {
        webhook_request = webhook_request.set(name, value);
    }

    match webhook_request.send_string(&payload_text) {
        Ok(response) => {
            let status = response.status();
            if (300..400).contains(&status) {
                AlertDeliveryReport {
                    event_id: event.id,
                    route_label: policy.provider.to_string(),
                    target,
                    status: AlertDeliveryStatus::Failed,
                    http_status: Some(status),
                    message: format!(
                        "webhook redirect rejected with HTTP {status}; redirects are disabled"
                    ),
                }
            } else {
                AlertDeliveryReport {
                    event_id: event.id,
                    route_label: policy.provider.to_string(),
                    target,
                    status: AlertDeliveryStatus::Delivered,
                    http_status: Some(status),
                    message: format!("webhook accepted alert with HTTP {status}"),
                }
            }
        }
        Err(ureq::Error::Status(status, response)) => {
            if (300..400).contains(&status) {
                return AlertDeliveryReport {
                    event_id: event.id,
                    route_label: policy.provider.to_string(),
                    target,
                    status: AlertDeliveryStatus::Failed,
                    http_status: Some(status),
                    message: format!(
                        "webhook redirect rejected with HTTP {status}; redirects are disabled"
                    ),
                };
            }
            let body = response.into_string().unwrap_or_default();
            let suffix = if body.trim().is_empty() {
                String::new()
            } else {
                format!(": {}", truncate_for_message(body.trim(), 160))
            };
            AlertDeliveryReport {
                event_id: event.id,
                route_label: policy.provider.to_string(),
                target,
                status: AlertDeliveryStatus::Failed,
                http_status: Some(status),
                message: format!("webhook rejected alert with HTTP {status}{suffix}"),
            }
        }
        Err(error) => AlertDeliveryReport {
            event_id: event.id,
            route_label: policy.provider.to_string(),
            target,
            status: AlertDeliveryStatus::Failed,
            http_status: None,
            message: format!("webhook delivery failed: {error}"),
        },
    }
}

fn skipped_delivery(event_id: i64, message: &str) -> AlertDeliveryReport {
    AlertDeliveryReport {
        event_id,
        route_label: "generic".to_string(),
        target: "none".to_string(),
        status: AlertDeliveryStatus::Skipped,
        http_status: None,
        message: message.to_string(),
    }
}

fn severity_rank(severity: EventSeverity) -> u8 {
    match severity {
        EventSeverity::Info => 0,
        EventSeverity::Warning => 1,
        EventSeverity::Critical => 2,
    }
}
