use eframe::egui;

pub(super) struct PluginPaneLayout {
    pub(super) catalog_width: f32,
    pub(super) activation_width: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

pub(super) fn plugin_pane_layout(available: egui::Vec2) -> PluginPaneLayout {
    let gap = 8.0;
    let catalog_width = (available.x * 0.48).clamp(340.0, 520.0);
    let activation_width = (available.x - catalog_width - gap).max(320.0);

    PluginPaneLayout {
        catalog_width,
        activation_width,
        height: available.y,
        gap,
    }
}
