use eframe::egui;

pub(super) struct OverviewLayout {
    pub(super) left_width: f32,
    pub(super) right_width: f32,
    pub(super) resource_height: f32,
    pub(super) fleet_height: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

pub(super) fn overview_layout(available: egui::Vec2) -> OverviewLayout {
    let gap = 8.0;
    let left_width = (available.x * 0.42).clamp(320.0, 460.0);
    let right_width = (available.x - left_width - gap).max(360.0);
    let resource_height = (available.y * 0.35).clamp(150.0, 190.0);
    let fleet_height = (available.y - resource_height - gap).max(220.0);

    OverviewLayout {
        left_width,
        right_width,
        resource_height,
        fleet_height,
        height: available.y,
        gap,
    }
}
