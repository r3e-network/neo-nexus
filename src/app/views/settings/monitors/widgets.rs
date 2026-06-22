use std::ops::RangeInclusive;

use eframe::egui;

use crate::app::theme;

pub(super) fn status_label(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

pub(super) fn interval_drag(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut u64,
    range: RangeInclusive<u64>,
    speed: f64,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(value)
                .range(range)
                .suffix(" s")
                .speed(speed),
        );
    });
}

pub(super) fn validation_error(ui: &mut egui::Ui, message: Option<&str>) {
    if let Some(message) = message {
        ui.label(egui::RichText::new(message).color(theme::danger()));
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/app/views/settings/monitors/widgets/tests.rs"]
mod tests;
