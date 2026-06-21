mod delivery;
mod payloads;
mod policy;
mod preview;
mod provider;
mod routing;
mod targets;
mod text;

pub use self::{
    delivery::{
        filter_alert_deliveries, AlertDelivery, AlertDeliveryFilter, AlertDeliveryReport,
        AlertDeliveryStatus,
    },
    payloads::{alert_provider_payload, alert_webhook_payload},
    policy::AlertRoutingPolicy,
    preview::{preview_alert_route, AlertPreviewHeader, AlertPreviewReport},
    provider::AlertProvider,
    routing::{deliver_webhook_alert, should_route_alert},
    targets::{alert_target_label, normalized_webhook_url},
};

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
use crate::events::{EventSeverity, RuntimeEvent};

#[cfg(test)]
use self::payloads::{
    alert_delivery_request, datadog_event_payload, opsgenie_alert_payload, pagerduty_alert_payload,
    telegram_alert_payload,
};
#[cfg(test)]
use self::targets::{datadog_target, opsgenie_target, pagerduty_target, telegram_target};

const DEFAULT_WEBHOOK_TIMEOUT_SECONDS: u64 = 5;
const MIN_WEBHOOK_TIMEOUT_SECONDS: u64 = 1;
const MAX_WEBHOOK_TIMEOUT_SECONDS: u64 = 30;
const MESSAGE_LIMIT: usize = 1_600;
const OPSGENIE_MESSAGE_LIMIT: usize = 130;

#[cfg(test)]
#[path = "../tests/unit/alerts/tests.rs"]
mod tests;
