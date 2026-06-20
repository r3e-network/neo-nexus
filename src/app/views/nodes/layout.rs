use eframe::egui;

pub(super) struct NodePaneLayout {
    pub(super) definition_width: f32,
    pub(super) selected_width: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

pub(super) fn node_pane_layout(available: egui::Vec2) -> NodePaneLayout {
    let gap = 8.0;
    let definition_width = (available.x * 0.62).clamp(400.0, 680.0);
    let selected_width = (available.x - definition_width - gap).max(300.0);

    NodePaneLayout {
        definition_width,
        selected_width,
        height: available.y,
        gap,
    }
}
