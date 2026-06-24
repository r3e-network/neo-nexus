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
    // Split the workspace into a left detail column and a right monitor column.
    // Both widths are derived from the available width so the two columns always
    // fit the CentralPanel and never overflow: the left is a 42% share clamped to
    // a sane range, and the right takes whatever remains. Floors are expressed as
    // a fraction of the available width rather than fixed pixels, so a narrow
    // window (e.g. all three side panels open) shrinks the columns gracefully
    // instead of forcing a fixed 360px right pane that overflows.
    let left_width = (available.x * 0.42).clamp(available.x * 0.30, available.x * 0.55);
    let right_width = (available.x - left_width - gap).max(available.x * 0.40);
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
