use eframe::egui;

pub(super) struct NodePaneLayout {
    pub(super) definition_width: f32,
    pub(super) selected_width: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

pub(super) fn node_pane_layout(available: egui::Vec2) -> NodePaneLayout {
    let gap = 8.0;
    // The definition form is the dominant pane (62% share); the selected-node
    // summary takes the remainder. Both are derived from the available width so
    // they always fit the CentralPanel — floors are a fraction of the available
    // width, not fixed pixels, so a narrow center (all side panels open) shrinks
    // the panes gracefully instead of forcing a fixed 400px definition pane that
    // overflows.
    let definition_width = (available.x * 0.62).clamp(available.x * 0.40, available.x * 0.72);
    let selected_width = (available.x - definition_width - gap).max(available.x * 0.26);

    NodePaneLayout {
        definition_width,
        selected_width,
        height: available.y,
        gap,
    }
}
