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
    visuals.hyperlink_color = palette.accent_text;
    // `selection` backs both text ranges and every `selectable_label`. At 30%
    // the coral composited into a hot block behind ordinary table text; the
    // pre-blended wash is the same tint the sidebar and list rows use, so all
    // selection in the workbench now reads as one quiet highlight.
    visuals.selection.bg_fill = palette.accent_wash;
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, palette.border_strong);
    visuals.window_stroke = egui::Stroke::new(1.0_f32, palette.border);

    let rounding = egui::CornerRadius::same(9);
    visuals.window_corner_radius = egui::CornerRadius::same(14);
    visuals.menu_corner_radius = egui::CornerRadius::same(10);
    // Flat surfaces: elevation is carried by fill tiers and hairlines, so no
    // surface in the workbench casts a shadow.
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.popup_shadow = egui::Shadow::NONE;

    // Quiet, flat surfaces: cards/buttons read as the same family with hairline
    // borders, and interaction is a tint rather than a solid accent block.
    visuals.widgets.noninteractive.corner_radius = rounding;
    visuals.widgets.noninteractive.bg_fill = palette.card_fill;
    visuals.widgets.noninteractive.weak_bg_fill = palette.card_fill;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, palette.border);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, palette.muted_text);

    visuals.widgets.inactive.corner_radius = rounding;
    visuals.widgets.inactive.bg_fill = palette.card_fill;
    visuals.widgets.inactive.weak_bg_fill = palette.card_fill;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, palette.border);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, palette.text);

    visuals.widgets.hovered.corner_radius = rounding;
    visuals.widgets.hovered.bg_fill = palette.faint_fill;
    visuals.widgets.hovered.weak_bg_fill = palette.faint_fill;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, palette.border_strong);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, palette.text);

    // `active` is the pressed state — and, critically, the source egui reads for
    // `Visuals::strong_text_color()`. Filling it with the accent used to make
    // every `.strong()` run (page titles, section titles, metric values, status
    // values) render in `on_accent` white and vanish on the light theme. The
    // pressed state is therefore a coral *wash* with normal-strength text, which
    // both looks right and keeps bold copy readable.
    visuals.widgets.active.corner_radius = rounding;
    visuals.widgets.active.bg_fill = palette.accent_wash;
    visuals.widgets.active.weak_bg_fill = palette.accent_wash;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, palette.accent);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, palette.text);

    visuals.widgets.open.corner_radius = rounding;
    visuals.widgets.open.bg_fill = palette.faint_fill;
    visuals.widgets.open.weak_bg_fill = palette.faint_fill;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, palette.border_strong);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, palette.text);

    style.spacing.item_spacing = egui::vec2(metrics.item_spacing_x, metrics.item_spacing_y);
    style.spacing.button_padding = egui::vec2(metrics.button_pad_x, metrics.button_pad_y);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.spacing.indent = 20.0;
    style.spacing.interact_size.y = metrics.interact_y;

    context.set_style(style);
}
