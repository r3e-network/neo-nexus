use eframe::egui;

use crate::app::theme;

use crate::app::{
    domain::RuntimeUpgradePolicy, theme::accent, widgets::labeled_combo, NeoNexusApp,
};

pub(super) fn render_policy_form(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let profiles = app.runtime_catalog_profiles.clone();
    let selected_profile = app
        .runtime_upgrade_policy_draft
        .catalog_profile_id
        .as_deref()
        .and_then(|id| profiles.iter().find(|profile| profile.id == id))
        .map_or_else(
            || "No catalog profile".to_string(),
            |profile| profile.label.clone(),
        );

    ui.checkbox(
        &mut app.runtime_upgrade_policy_draft.enabled,
        "Schedule catalog upgrades for fleet nodes",
    );
    labeled_combo(
        ui,
        "Catalog",
        "runtime_upgrade_policy_catalog_profile",
        selected_profile,
        |ui| {
            if profiles.is_empty() {
                ui.label("No saved catalog profiles");
            }
            for profile in profiles {
                ui.selectable_value(
                    &mut app.runtime_upgrade_policy_draft.catalog_profile_id,
                    Some(profile.id),
                    profile.label,
                );
            }
        },
    );
    ui.checkbox(
        &mut app.runtime_upgrade_policy_draft.require_signed_catalog,
        "Require signed catalog",
    );
    ui.separator();

    render_timing_controls(app, ui);
    render_validation_state(app, ui);
}

fn render_timing_controls(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Interval");
        ui.add(
            egui::DragValue::new(&mut app.runtime_upgrade_policy_draft.interval_minutes)
                .range(
                    RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES
                        ..=RuntimeUpgradePolicy::MAX_INTERVAL_MINUTES,
                )
                .suffix(" min")
                .speed(15.0),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Max nodes");
        ui.add(
            egui::DragValue::new(&mut app.runtime_upgrade_policy_draft.max_nodes_per_run)
                .range(1..=RuntimeUpgradePolicy::MAX_NODES_PER_RUN)
                .speed(1.0),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Wave delay");
        ui.add(
            egui::DragValue::new(&mut app.runtime_upgrade_policy_draft.wave_delay_minutes)
                .range(0..=RuntimeUpgradePolicy::MAX_WAVE_DELAY_MINUTES)
                .suffix(" min")
                .speed(15.0),
        );
    });
    ui.checkbox(
        &mut app.runtime_upgrade_policy_draft.maintenance_window_enabled,
        "Use maintenance window",
    );
    ui.horizontal(|ui| {
        ui.label("Window UTC");
        ui.add_enabled(
            app.runtime_upgrade_policy_draft.maintenance_window_enabled,
            egui::DragValue::new(
                &mut app
                    .runtime_upgrade_policy_draft
                    .maintenance_window_start_minute_utc,
            )
            .range(0..=RuntimeUpgradePolicy::MINUTES_PER_DAY - 1)
            .suffix(" min")
            .speed(15.0),
        );
        ui.label("to");
        ui.add_enabled(
            app.runtime_upgrade_policy_draft.maintenance_window_enabled,
            egui::DragValue::new(
                &mut app
                    .runtime_upgrade_policy_draft
                    .maintenance_window_end_minute_utc,
            )
            .range(0..=RuntimeUpgradePolicy::MINUTES_PER_DAY - 1)
            .suffix(" min")
            .speed(15.0),
        );
    });
}

fn render_validation_state(app: &NeoNexusApp, ui: &mut egui::Ui) {
    ui.add_space(theme::SM);
    if let Some(message) = app.runtime_upgrade_policy_draft.validation_message() {
        ui.label(egui::RichText::new(message).color(theme::danger()));
    } else {
        ui.label(egui::RichText::new("Policy draft is valid.").color(accent()));
    }
}
