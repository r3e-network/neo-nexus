use eframe::egui;

pub(super) struct LogLayout {
    pub(super) context_width: f32,
    pub(super) output_width: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

pub(super) fn log_layout(available: egui::Vec2) -> LogLayout {
    let gap = 8.0;
    let context_width = (available.x * 0.34).clamp(300.0, 430.0);
    let output_width = (available.x - context_width - gap).max(420.0);

    LogLayout {
        context_width,
        output_width,
        height: available.y,
        gap,
    }
}
