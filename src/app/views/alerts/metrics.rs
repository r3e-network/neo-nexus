use eframe::egui;

use crate::alerts::{alert_target_label, AlertDelivery, AlertDeliveryStatus, AlertRoutingPolicy};

use super::super::super::{text::truncate_middle, widgets::metric_tile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AlertDeliverySummary {
    delivered: usize,
    failed: usize,
}

pub(super) fn alert_delivery_summary(deliveries: &[AlertDelivery]) -> AlertDeliverySummary {
    let failed = deliveries
        .iter()
        .filter(|delivery| delivery.status == AlertDeliveryStatus::Failed)
        .count();
    let delivered = deliveries
        .iter()
        .filter(|delivery| delivery.status == AlertDeliveryStatus::Delivered)
        .count();

    AlertDeliverySummary { delivered, failed }
}

pub(super) fn render_alert_metrics(
    ui: &mut egui::Ui,
    policy: &AlertRoutingPolicy,
    pending: usize,
    summary: AlertDeliverySummary,
) {
    let target = policy
        .webhook_url
        .as_deref()
        .map(alert_target_label)
        .unwrap_or_else(|| "not configured".to_string());

    ui.horizontal(|ui| {
        metric_tile(
            ui,
            "Routing",
            if policy.enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            &format!("{}+ events", policy.min_severity),
        );
        metric_tile(
            ui,
            "Target",
            &truncate_middle(&target, 18),
            "webhook endpoint",
        );
        metric_tile(ui, "Pending", &pending.to_string(), "background sends");
        metric_tile(
            ui,
            "Recent",
            &format!("{}/{}", summary.delivered, summary.failed),
            "delivered/failed",
        );
    });
}
