use eframe::egui;

pub(super) struct MonitorLayout {
    pub(super) left_width: f32,
    pub(super) process_width: f32,
    pub(super) pressure_height: f32,
    pub(super) telemetry_height: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

pub(super) fn monitor_layout(available: egui::Vec2) -> MonitorLayout {
    let gap = 8.0;
    let left_width = (available.x * 0.42).clamp(340.0, 500.0);
    let pressure_height = (available.y * 0.52).clamp(230.0, 310.0);
    let telemetry_height = (available.y - pressure_height - gap).max(150.0);
    let process_width = (available.x - left_width - gap).max(460.0);

    MonitorLayout {
        left_width,
        process_width,
        pressure_height,
        telemetry_height,
        height: available.y,
        gap,
    }
}
