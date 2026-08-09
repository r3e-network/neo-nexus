use eframe::egui;

use crate::app::{
    domain::{alert_target_label, AlertRoutingPolicy},
    theme::{accent_text, muted_text},
    AlertRoutingPolicyDraft,
};

/// The route as saved, in one line.
///
/// This used to be five facts — Active, Threshold, Provider, Timeout, Target —
/// stacked directly above a form that edits those same five fields. Two copies of
/// one thing, ~110pt of a pane that does not scroll, and no way to tell which was
/// live. The form below is the editor; this is only here to answer "what is
/// actually routing right now, and does the draft differ from it".
pub(super) fn render_policy_status(
    ui: &mut egui::Ui,
    policy: &AlertRoutingPolicy,
    draft: &AlertRoutingPolicyDraft,
) {
    let target = policy
        .webhook_url
        .as_deref()
        .map(alert_target_label)
        .unwrap_or_else(|| "no target".to_string());
    let summary = if policy.enabled {
        format!(
            "{} · {} and above · {}s · {target}",
            policy.provider.display_name(),
            policy.min_severity,
            policy.timeout_seconds,
        )
    } else {
        "No route is active — nothing is delivered.".to_string()
    };

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Saved").color(muted_text()));
        ui.label(egui::RichText::new(summary).strong());
    });
    if draft.differs_from(policy) {
        ui.label(egui::RichText::new("Unsaved changes below.").color(accent_text()));
    }
}
