use crate::app::domain::{AlertProvider, AlertRoutingPolicy, EventSeverity};

use super::super::text::optional_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct AlertRoutingPolicyDraft {
    pub(in crate::app) enabled: bool,
    pub(in crate::app) provider: AlertProvider,
    pub(in crate::app) min_severity: EventSeverity,
    pub(in crate::app) webhook_url: String,
    pub(in crate::app) timeout_seconds: u64,
}

impl AlertRoutingPolicyDraft {
    pub(in crate::app) fn from_policy(policy: &AlertRoutingPolicy) -> Self {
        Self {
            enabled: policy.enabled,
            provider: policy.provider,
            min_severity: policy.min_severity,
            webhook_url: policy.webhook_url.clone().unwrap_or_default(),
            timeout_seconds: policy.timeout_seconds,
        }
    }

    pub(in crate::app) fn to_policy(&self) -> AlertRoutingPolicy {
        AlertRoutingPolicy {
            enabled: self.enabled,
            provider: self.provider,
            min_severity: self.min_severity,
            webhook_url: optional_text(&self.webhook_url),
            timeout_seconds: self.timeout_seconds,
        }
        .normalized()
    }

    pub(in crate::app) fn validation_message(&self) -> Option<String> {
        AlertRoutingPolicy {
            enabled: self.enabled,
            provider: self.provider,
            min_severity: self.min_severity,
            webhook_url: optional_text(&self.webhook_url),
            timeout_seconds: self.timeout_seconds,
        }
        .validation_message()
    }

    pub(in crate::app) fn differs_from(&self, policy: &AlertRoutingPolicy) -> bool {
        self != &Self::from_policy(policy)
    }
}
