use std::ops::RangeInclusive;

use eframe::egui;

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
        ui.label(egui::RichText::new(message).color(egui::Color32::from_rgb(185, 28, 28)));
    }
}

#[cfg(test)]
mod tests {
    use super::status_label;

    #[test]
    fn monitor_status_label_matches_policy_enabled_state() {
        assert_eq!(status_label(true), "enabled");
        assert_eq!(status_label(false), "disabled");
    }
}
