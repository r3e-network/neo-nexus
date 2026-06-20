use std::{fmt, str::FromStr};

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertDeliveryStatus {
    Delivered,
    Failed,
    Skipped,
}

impl AlertDeliveryStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

impl fmt::Display for AlertDeliveryStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

impl FromStr for AlertDeliveryStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "delivered" => Ok(Self::Delivered),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            other => anyhow::bail!("unsupported alert delivery status: {other}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertDelivery {
    pub id: i64,
    pub event_id: i64,
    pub attempted_at_unix: u64,
    pub route_label: String,
    pub target: String,
    pub status: AlertDeliveryStatus,
    pub http_status: Option<u16>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertDeliveryReport {
    pub event_id: i64,
    pub route_label: String,
    pub target: String,
    pub status: AlertDeliveryStatus,
    pub http_status: Option<u16>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AlertDeliveryFilter {
    pub status: Option<AlertDeliveryStatus>,
    pub query: String,
}

impl AlertDeliveryFilter {
    pub fn new(status: Option<AlertDeliveryStatus>, query: impl Into<String>) -> Self {
        Self {
            status,
            query: query.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.status.is_none() && self.query.trim().is_empty()
    }
}

pub fn filter_alert_deliveries(
    deliveries: &[AlertDelivery],
    filter: &AlertDeliveryFilter,
) -> Vec<AlertDelivery> {
    let query = filter.query.trim().to_lowercase();
    deliveries
        .iter()
        .filter(|delivery| status_matches(delivery, filter.status))
        .filter(|delivery| query.is_empty() || query_matches(delivery, &query))
        .cloned()
        .collect()
}

pub fn filter_alert_deliveries_by_status(
    deliveries: &[AlertDelivery],
    status: Option<AlertDeliveryStatus>,
) -> Vec<AlertDelivery> {
    filter_alert_deliveries(deliveries, &AlertDeliveryFilter::new(status, ""))
}

fn status_matches(delivery: &AlertDelivery, status: Option<AlertDeliveryStatus>) -> bool {
    status.is_none_or(|status| delivery.status == status)
}

fn query_matches(delivery: &AlertDelivery, query: &str) -> bool {
    text_matches(delivery.route_label.as_str(), query)
        || text_matches(delivery.target.as_str(), query)
        || text_matches(delivery.message.as_str(), query)
        || text_matches(delivery.status.label(), query)
        || text_matches(&delivery.id.to_string(), query)
        || text_matches(&delivery.event_id.to_string(), query)
        || http_matches(delivery.http_status, query)
}

fn http_matches(status: Option<u16>, query: &str) -> bool {
    status.is_some_and(|status| {
        let status = status.to_string();
        text_matches(&status, query) || text_matches(&format!("http {status}"), query)
    })
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}
