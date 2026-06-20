use anyhow::Result;
use serde_json::Value;

use crate::events::RuntimeEvent;

use super::{
    targets::{datadog_target, opsgenie_target, pagerduty_target, telegram_target},
    AlertProvider,
};

mod chat;
mod common;
mod generic;
mod incident;

#[cfg(test)]
pub(super) use self::{
    chat::telegram_alert_payload,
    incident::{datadog_event_payload, opsgenie_alert_payload, pagerduty_alert_payload},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AlertDeliveryRequest {
    pub(super) endpoint_url: String,
    pub(super) payload: Value,
    pub(super) headers: Vec<(String, String)>,
}

pub fn alert_webhook_payload(event: &RuntimeEvent, application_version: &str) -> Value {
    generic::generic_alert_payload(event, application_version)
}

pub fn alert_provider_payload(
    provider: AlertProvider,
    event: &RuntimeEvent,
    application_version: &str,
) -> Value {
    match provider {
        AlertProvider::Generic => generic::generic_alert_payload(event, application_version),
        AlertProvider::Slack => chat::slack_alert_payload(event, application_version),
        AlertProvider::Discord => chat::discord_alert_payload(event, application_version),
        AlertProvider::Telegram => chat::telegram_alert_payload(event, application_version, ""),
        AlertProvider::PagerDuty => {
            incident::pagerduty_alert_payload(event, application_version, "")
        }
        AlertProvider::Opsgenie => incident::opsgenie_alert_payload(event, application_version),
        AlertProvider::Datadog => incident::datadog_event_payload(event, application_version),
    }
}

pub(super) fn alert_delivery_request(
    provider: AlertProvider,
    event: &RuntimeEvent,
    application_version: &str,
    target_url: &str,
) -> Result<AlertDeliveryRequest> {
    let mut endpoint_url = target_url.to_string();
    let mut headers = Vec::new();
    let payload = match provider {
        AlertProvider::Generic => generic::generic_alert_payload(event, application_version),
        AlertProvider::Slack => chat::slack_alert_payload(event, application_version),
        AlertProvider::Discord => chat::discord_alert_payload(event, application_version),
        AlertProvider::Telegram => {
            let target = telegram_target(target_url)?;
            chat::telegram_alert_payload(event, application_version, &target.chat_id)
        }
        AlertProvider::PagerDuty => {
            let target = pagerduty_target(target_url)?;
            incident::pagerduty_alert_payload(event, application_version, &target.routing_key)
        }
        AlertProvider::Opsgenie => {
            let target = opsgenie_target(target_url)?;
            endpoint_url = target.endpoint_url;
            headers.push((
                "Authorization".to_string(),
                format!("GenieKey {}", target.api_key),
            ));
            incident::opsgenie_alert_payload(event, application_version)
        }
        AlertProvider::Datadog => {
            let target = datadog_target(target_url)?;
            endpoint_url = target.endpoint_url;
            headers.push(("DD-API-KEY".to_string(), target.api_key));
            incident::datadog_event_payload(event, application_version)
        }
    };
    Ok(AlertDeliveryRequest {
        endpoint_url,
        payload,
        headers,
    })
}
