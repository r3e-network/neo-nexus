use eframe::egui;

pub(super) struct WalletPaneLayout {
    pub(super) import_width: f32,
    pub(super) registry_width: f32,
    pub(super) registry_height: f32,
    pub(super) height: f32,
    pub(super) gap: f32,
}

pub(super) fn wallet_pane_layout(available: egui::Vec2) -> WalletPaneLayout {
    let gap = 8.0;
    let import_width = (available.x * 0.38)
        .clamp(320.0, 440.0)
        .min((available.x - gap - 340.0).max(320.0));
    let registry_width = (available.x - import_width - gap).max(340.0);
    let registry_height = (available.y * 0.52)
        .clamp(244.0, 328.0)
        .min((available.y - 210.0).max(244.0));

    WalletPaneLayout {
        import_width,
        registry_width,
        registry_height,
        height: available.y,
        gap,
    }
}
