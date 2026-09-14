use std::time::Duration;

use crate::events::{EventKind, EventSeverity};

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
    /// Which kinds of event are worth waking someone for. Empty means all.
    ///
    /// A severity floor was the *only* condition this policy could express, so
    /// "page me when a node stalls but not when a plugin is installed" was
    /// unsayable — and severity is assigned at the call site, one global
    /// opinion for the whole product.
    pub kinds: Vec<EventKind>,
    /// Which nodes. Empty means all.
    ///
    /// "Page on the validator, warn on the observers" is the most ordinary
    /// routing rule a node operator has, and it could not be expressed at all.
    pub node_ids: Vec<String>,
}

impl Default for AlertRoutingPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AlertProvider::Generic,
            min_severity: EventSeverity::Warning,
            webhook_url: None,
            timeout_seconds: DEFAULT_WEBHOOK_TIMEOUT_SECONDS,
            kinds: Vec::new(),
            node_ids: Vec::new(),
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
        self.kinds.sort_by_key(|kind| kind.label());
        self.kinds.dedup();
        self.node_ids.retain(|id| !id.trim().is_empty());
        self.node_ids.sort();
        self.node_ids.dedup();
        self
    }

    /// Whether this policy names any scope beyond the severity floor.
    pub fn is_scoped(&self) -> bool {
        !self.kinds.is_empty() || !self.node_ids.is_empty()
    }

    /// What the scope excludes, for an operator reading the settings page.
    ///
    /// A narrowed route that does not say what it narrowed to is how an alert
    /// rule silently stops covering the thing it was written for.
    pub fn scope_description(&self) -> String {
        let mut parts = Vec::new();
        if self.kinds.is_empty() {
            parts.push("every kind of event".to_string());
        } else {
            parts.push(format!(
                "only {}",
                self.kinds
                    .iter()
                    .map(|kind| kind.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if self.node_ids.is_empty() {
            parts.push("from any node".to_string());
        } else {
            parts.push(format!("from {} selected node(s)", self.node_ids.len()));
        }
        parts.join(", ")
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
                "enabled for {}+ events via {} {} — {}",
                self.min_severity,
                self.provider.display_name(),
                target,
                self.scope_description(),
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
