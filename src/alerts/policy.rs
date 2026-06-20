use std::time::Duration;

use crate::events::EventSeverity;

use super::{
    targets::{alert_target_label, normalized_webhook_url, validate_provider_target},
    AlertProvider, DEFAULT_WEBHOOK_TIMEOUT_SECONDS, MAX_WEBHOOK_TIMEOUT_SECONDS,
    MIN_WEBHOOK_TIMEOUT_SECONDS,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertRoutingPolicy {
    pub enabled: bool,
    pub provider: AlertProvider,
    pub min_severity: EventSeverity,
    pub webhook_url: Option<String>,
    pub timeout_seconds: u64,
}

impl Default for AlertRoutingPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AlertProvider::Generic,
            min_severity: EventSeverity::Warning,
            webhook_url: None,
            timeout_seconds: DEFAULT_WEBHOOK_TIMEOUT_SECONDS,
        }
    }
}

impl AlertRoutingPolicy {
    pub const MIN_TIMEOUT_SECONDS: u64 = MIN_WEBHOOK_TIMEOUT_SECONDS;
    pub const MAX_TIMEOUT_SECONDS: u64 = MAX_WEBHOOK_TIMEOUT_SECONDS;

    pub fn normalized(mut self) -> Self {
        self.timeout_seconds = self
            .timeout_seconds
            .clamp(Self::MIN_TIMEOUT_SECONDS, Self::MAX_TIMEOUT_SECONDS);
        self.webhook_url = self
            .webhook_url
            .and_then(|url| normalized_webhook_url(&url).ok());
        self
    }

    pub fn validation_message(&self) -> Option<String> {
        if self.enabled && self.webhook_url.as_deref().unwrap_or("").trim().is_empty() {
            return Some("Enabled alert routing requires a webhook URL".to_string());
        }
        if let Some(url) = self.webhook_url.as_deref() {
            if let Err(error) = normalized_webhook_url(url) {
                return Some(error.to_string());
            }
            if let Err(error) = validate_provider_target(self.provider, url) {
                return Some(error.to_string());
            }
        }
        if self.timeout_seconds < Self::MIN_TIMEOUT_SECONDS {
            return Some("Alert webhook timeout is too short".to_string());
        }
        if self.timeout_seconds > Self::MAX_TIMEOUT_SECONDS {
            return Some("Alert webhook timeout is too long".to_string());
        }
        None
    }

    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    pub fn describe(&self) -> String {
        if self.enabled {
            let target = self
                .webhook_url
                .as_deref()
                .map(alert_target_label)
                .unwrap_or_else(|| "no target".to_string());
            format!(
                "enabled for {}+ events via {} {}",
                self.min_severity,
                self.provider.display_name(),
                target,
            )
        } else {
            format!(
                "disabled; {} threshold {}",
                self.provider.display_name(),
                self.min_severity
            )
        }
    }
}
