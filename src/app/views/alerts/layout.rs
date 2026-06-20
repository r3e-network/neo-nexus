use eframe::egui;

pub(super) struct AlertPaneLayout {
    pub(super) policy_width: f32,
    pub(super) history_width: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

pub(super) fn alert_pane_layout(available: egui::Vec2) -> AlertPaneLayout {
    let gap = 8.0;
    let policy_width = (available.x * 0.38)
        .clamp(340.0, 500.0)
        .min((available.x - gap - 360.0).max(340.0));
    let history_width = (available.x - policy_width - gap).max(360.0);

    AlertPaneLayout {
        policy_width,
        history_width,
        height: available.y,
        gap,
    }
}
