use super::*;

/// Apply palette + density control metrics. List row heights are read by
/// widgets from [`DensityMetrics`], not from egui Style.
pub(in crate::app) fn configure_style_with_density(
    context: &egui::Context,
    theme: Theme,
    density: UiDensity,
) {
    set_active_theme(theme);
    let palette = palette(theme);
    let metrics = DensityMetrics::for_density(density);
    let mut style = (*context.style()).clone();

    style.visuals = if theme.is_dark() {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    let visuals = &mut style.visuals;
    // The central workspace is the deepest background tier (the canvas cards
    // float on); the chrome panels (sidebar, header, inspector) lift off it by
    // carrying their own lighter `panel_fill` frame set in `frame.rs`.
    visuals.panel_fill = palette.window_fill;
    visuals.window_fill = palette.window_fill;
    visuals.extreme_bg_color = palette.field_fill;
    visuals.faint_bg_color = palette.faint_fill;
    visuals.override_text_color = Some(palette.text);
    visuals.hyperlink_color = palette.accent;
    visuals.selection.bg_fill = palette.accent.gamma_multiply(0.30);
    visuals.selection.stroke = egui::Stroke::new(1.0, palette.accent);
    visuals.window_stroke = egui::Stroke::new(1.0, palette.border);

    let rounding = egui::CornerRadius::same(8);
    visuals.window_corner_radius = egui::CornerRadius::same(12);
    visuals.menu_corner_radius = egui::CornerRadius::same(10);

    // Quiet, flat surfaces: cards/buttons read as the same family with hairline
    // borders, and only hover/active states pick up the accent.
    visuals.widgets.noninteractive.corner_radius = rounding;
    visuals.widgets.noninteractive.bg_fill = palette.card_fill;
    visuals.widgets.noninteractive.weak_bg_fill = palette.card_fill;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, palette.border);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, palette.muted_text);

    visuals.widgets.inactive.corner_radius = rounding;
    visuals.widgets.inactive.bg_fill = palette.card_fill;
    visuals.widgets.inactive.weak_bg_fill = palette.card_fill;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, palette.border);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, palette.text);

    visuals.widgets.hovered.corner_radius = rounding;
    visuals.widgets.hovered.bg_fill = palette.faint_fill;
    visuals.widgets.hovered.weak_bg_fill = palette.faint_fill;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, palette.accent);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, palette.text);

    visuals.widgets.active.corner_radius = rounding;
    visuals.widgets.active.bg_fill = palette.accent;
    visuals.widgets.active.weak_bg_fill = palette.accent;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, palette.accent_hover);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, palette.on_accent);

    visuals.widgets.open.corner_radius = rounding;
    visuals.widgets.open.bg_fill = palette.faint_fill;
    visuals.widgets.open.weak_bg_fill = palette.faint_fill;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, palette.border);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, palette.text);

    style.spacing.item_spacing = egui::vec2(metrics.item_spacing_x, metrics.item_spacing_y);
    style.spacing.button_padding = egui::vec2(metrics.button_pad_x, metrics.button_pad_y);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.spacing.indent = 20.0;
    style.spacing.interact_size.y = metrics.interact_y;

    context.set_style(style);
}
