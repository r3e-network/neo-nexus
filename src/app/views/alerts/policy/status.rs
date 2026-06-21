use eframe::egui;

use crate::app::{
    domain::{alert_target_label, AlertRoutingPolicy},
    widgets::fact,
};

pub(super) fn render_policy_status(ui: &mut egui::Ui, policy: &AlertRoutingPolicy) {
    fact(
        ui,
        "Active",
        if policy.enabled {
            "enabled"
        } else {
            "disabled"
        },
    );
    fact(ui, "Threshold", &policy.min_severity.to_string());
    fact(ui, "Provider", policy.provider.display_name());
    fact(ui, "Timeout", &format!("{}s", policy.timeout_seconds));
    fact(
        ui,
        "Target",
        &policy
            .webhook_url
            .as_deref()
            .map(alert_target_label)
            .unwrap_or_else(|| "-".to_string()),
    );
}
