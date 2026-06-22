use eframe::egui;

use crate::app::domain::{
    alert_target_label, AlertDelivery, AlertDeliveryStatus, AlertRoutingPolicy,
};

use super::super::super::{text::truncate_middle, widgets::metric_row};

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

    let routing_caption = format!("{}+ events", policy.min_severity);
    let target_short = truncate_middle(&target, 18);
    let pending_label = pending.to_string();
    let recent = format!("{}/{}", summary.delivered, summary.failed);
    metric_row(
        ui,
        &[
            (
                "Routing",
                if policy.enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                &routing_caption,
            ),
            ("Target", &target_short, "webhook endpoint"),
            ("Pending", &pending_label, "background sends"),
            ("Recent", &recent, "delivered/failed"),
        ],
    );
}
