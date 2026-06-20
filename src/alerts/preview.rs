use std::fmt::Write;

use anyhow::Result;
use serde::Serialize;

use crate::{
    events::RuntimeEvent,
    redaction::{redact_sensitive_text, REDACTED_VALUE},
};

use super::{payloads::alert_delivery_request, targets::alert_target_label, AlertProvider};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AlertPreviewHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AlertPreviewReport {
    pub status: &'static str,
    pub provider: String,
    pub severity: String,
    pub target: String,
    pub endpoint: String,
    pub header_count: usize,
    pub headers: Vec<AlertPreviewHeader>,
    pub payload_json: String,
}

impl AlertPreviewReport {
    pub fn to_cli_text(&self) -> String {
        let mut output = String::new();
        let _ = writeln!(&mut output, "alert-preview: {}", self.status);
        let _ = writeln!(&mut output, "provider: {}", self.provider);
        let _ = writeln!(&mut output, "severity: {}", self.severity);
        let _ = writeln!(&mut output, "target: {}", self.target);
        let _ = writeln!(&mut output, "endpoint: {}", self.endpoint);
        let _ = writeln!(&mut output, "headers: {}", self.header_count);
        for header in &self.headers {
            let _ = writeln!(&mut output, "header: {}={}", header.name, header.value);
        }
        let _ = writeln!(&mut output, "payload-json:");
        let _ = writeln!(&mut output, "{}", self.payload_json);
        output
    }
}

pub fn preview_alert_route(
    provider: AlertProvider,
    target_url: &str,
    event: &RuntimeEvent,
    application_version: &str,
) -> Result<AlertPreviewReport> {
    let request = alert_delivery_request(provider, event, application_version, target_url)?;
    let payload_json = redact_sensitive_text(&serde_json::to_string_pretty(&request.payload)?);
    let headers = request
        .headers
        .into_iter()
        .map(|(name, value)| AlertPreviewHeader {
            name,
            value: if value.is_empty() {
                String::new()
            } else {
                REDACTED_VALUE.to_string()
            },
        })
        .collect::<Vec<_>>();

    Ok(AlertPreviewReport {
        status: "ready",
        provider: provider.label().to_string(),
        severity: event.severity.label().to_string(),
        target: alert_target_label(target_url),
        endpoint: alert_target_label(&request.endpoint_url),
        header_count: headers.len(),
        headers,
        payload_json,
    })
}
