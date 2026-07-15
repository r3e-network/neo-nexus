use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn preview_alert_routing_policy_draft(&mut self) {
        if let Some(message) = self.async_bus.alert_routing_policy_draft.validation_message() {
            self.async_bus.last_alert_preview = None;
            self.async_bus.last_alert_preview_policy = None;
            self.session.notice = Some(message);
            return;
        }

        let policy = self.async_bus.alert_routing_policy_draft.to_policy();
        let Some(target_url) = policy.webhook_url.as_deref() else {
            self.async_bus.last_alert_preview = None;
            self.async_bus.last_alert_preview_policy = None;
            self.session.notice = Some("Alert preview requires a target URL".to_string());
            return;
        };
        let event = RuntimeEvent {
            id: 0,
            occurred_at_unix: match current_unix_time() {
                Ok(timestamp) => timestamp,
                Err(error) => {
                    self.async_bus.last_alert_preview = None;
                    self.async_bus.last_alert_preview_policy = None;
                    self.session.notice = Some(error.to_string());
                    return;
                }
            },
            node_id: None,
            node_name: Some("alert-preview".to_string()),
            kind: EventKind::AlertRoutingPolicyUpdated,
            severity: policy.min_severity,
            message: "Operator alert route preview".to_string(),
        };

        match preview_alert_route(
            policy.provider,
            target_url,
            &event,
            env!("CARGO_PKG_VERSION"),
        ) {
            Ok(report) => {
                self.session.notice = Some(format!(
                    "Alert preview ready: {} route to {}",
                    report.provider, report.endpoint
                ));
                self.async_bus.last_alert_preview = Some(report);
                self.async_bus.last_alert_preview_policy = Some(policy);
            }
            Err(error) => {
                self.async_bus.last_alert_preview = None;
                self.async_bus.last_alert_preview_policy = None;
                self.session.notice = Some(format!("Alert preview failed: {error}"));
            }
        }
    }

    pub(in crate::app) fn alert_preview_matches_draft(&self) -> bool {
        self.async_bus.last_alert_preview_policy
            .as_ref()
            .is_some_and(|policy| {
                self.async_bus.alert_routing_policy_draft
                    .validation_message()
                    .is_none()
                    && self.async_bus.alert_routing_policy_draft.to_policy() == policy.clone()
            })
    }

    pub(in crate::app) fn save_alert_routing_policy(&mut self) {
        if let Some(message) = self.async_bus.alert_routing_policy_draft.validation_message() {
            self.session.notice = Some(message);
            return;
        }

        let policy = self.async_bus.alert_routing_policy_draft.to_policy();
        match self.repository.save_alert_routing_policy(policy.clone()) {
            Ok(()) => {
                self.async_bus.alert_routing_policy = policy.normalized();
                self.async_bus.alert_routing_policy_draft =
                    AlertRoutingPolicyDraft::from_policy(&self.async_bus.alert_routing_policy);
                let message = format!(
                    "Alert routing policy saved: {}",
                    self.async_bus.alert_routing_policy.describe()
                );
                self.record_event_notice(
                    EventKind::AlertRoutingPolicyUpdated,
                    EventSeverity::Info,
                    message,
                );
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn reset_alert_routing_policy_draft(&mut self) {
        self.async_bus.alert_routing_policy_draft =
            AlertRoutingPolicyDraft::from_policy(&self.async_bus.alert_routing_policy);
        self.session.notice = Some("Alert routing policy draft reset".to_string());
    }

    pub(in crate::app) fn prune_alert_delivery_history(&mut self) {
        match self
            .repository
            .prune_alert_deliveries_keep_recent(ALERT_DELIVERY_RETAIN)
        {
            Ok(deleted) => {
                let message = format!(
                    "Alert delivery history pruned: {deleted} removed, retaining {ALERT_DELIVERY_RETAIN}"
                );
                self.record_event_notice(EventKind::EventsPruned, EventSeverity::Info, message);
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
