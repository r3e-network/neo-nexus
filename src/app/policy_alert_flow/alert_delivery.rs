use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn drain_alert_delivery_results(&mut self) {
        while let Ok(report) = self.alert_delivery_results.try_recv() {
            self.alert_delivery_pending = self.alert_delivery_pending.saturating_sub(1);
            let failed = report.status != crate::alerts::AlertDeliveryStatus::Delivered;
            let message = report.message.clone();
            if let Err(error) = self.repository.record_alert_delivery(&report) {
                self.notice = Some(error.to_string());
                continue;
            }
            if let Err(error) = self
                .repository
                .prune_alert_deliveries_keep_recent(ALERT_DELIVERY_RETAIN)
            {
                self.notice = Some(format!("{message}; alert history pruning failed: {error}"));
                continue;
            }
            if failed {
                self.notice = Some(message);
            }
        }
    }

    pub(in crate::app) fn route_alert_for_event(&mut self, event: RuntimeEvent) {
        if !should_route_alert(&self.alert_routing_policy, &event) {
            return;
        }

        let policy = self.alert_routing_policy.clone();
        let sender = self.alert_delivery_sender.clone();
        let thread_policy = policy.clone();
        let thread_event = event.clone();
        self.alert_delivery_pending += 1;
        if let Err(error) = thread::Builder::new()
            .name(format!("neonexus-alert-event-{}", event.id))
            .spawn(move || {
                let report =
                    deliver_webhook_alert(&thread_policy, &thread_event, env!("CARGO_PKG_VERSION"));
                let _ = sender.send(report);
            })
        {
            self.alert_delivery_pending = self.alert_delivery_pending.saturating_sub(1);
            let target = policy
                .webhook_url
                .as_deref()
                .map(alert_target_label)
                .unwrap_or_else(|| "webhook".to_string());
            let report = AlertDeliveryReport {
                event_id: event.id,
                route_label: policy.provider.to_string(),
                target,
                status: crate::alerts::AlertDeliveryStatus::Failed,
                http_status: None,
                message: format!("Unable to start alert delivery: {error}"),
            };
            if let Err(error) = self.repository.record_alert_delivery(&report) {
                self.notice = Some(error.to_string());
            } else {
                self.notice = Some(report.message);
            }
        }
    }
}
