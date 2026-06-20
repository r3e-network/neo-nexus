use crate::events::{EventSeverity, RuntimeEvent};

use super::{
    payloads::alert_delivery_request, targets::alert_target_label, text::truncate_for_message,
    AlertDeliveryReport, AlertDeliveryStatus, AlertRoutingPolicy,
};

pub fn should_route_alert(policy: &AlertRoutingPolicy, event: &RuntimeEvent) -> bool {
    policy.enabled
        && policy
            .webhook_url
            .as_deref()
            .is_some_and(|url| !url.is_empty())
        && severity_rank(event.severity) >= severity_rank(policy.min_severity)
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

    let mut webhook_request = ureq::post(&request.endpoint_url)
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
            AlertDeliveryReport {
                event_id: event.id,
                route_label: policy.provider.to_string(),
                target,
                status: AlertDeliveryStatus::Delivered,
                http_status: Some(status),
                message: format!("webhook accepted alert with HTTP {status}"),
            }
        }
        Err(ureq::Error::Status(status, response)) => {
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
