//! Counting what the delivery history holds.
//!
//! This module used to render a page-level metric row as well. Those tiles
//! restated the route the editor already shows, and the two figures that were
//! its own describe the history list, so they moved into its header.

use crate::app::domain::{AlertDelivery, AlertDeliveryStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AlertDeliverySummary {
    pub(super) delivered: usize,
    pub(super) failed: usize,
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
