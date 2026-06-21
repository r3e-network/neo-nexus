use super::*;

#[test]
fn theme_toggles_between_light_and_dark() {
    assert_eq!(Theme::Light.toggled(), Theme::Dark);
    assert_eq!(Theme::Dark.toggled(), Theme::Light);
    assert!(!Theme::Light.is_dark());
    assert!(Theme::Dark.is_dark());
}

#[test]
fn theme_round_trips_through_dark_mode_flag() {
    assert_eq!(Theme::from_dark_mode(false), Theme::Light);
    assert_eq!(Theme::from_dark_mode(true), Theme::Dark);
    assert!(!Theme::default().is_dark());
}

#[test]
fn toggle_label_names_the_target_theme() {
    assert_eq!(Theme::Light.toggle_label(), "Dark theme");
    assert_eq!(Theme::Dark.toggle_label(), "Light theme");
}

#[test]
fn dark_palette_differs_from_light_palette() {
    let light = palette(Theme::Light);
    let dark = palette(Theme::Dark);
    assert_ne!(light.window_fill, dark.window_fill);
    assert_ne!(light.card_fill, dark.card_fill);
    assert_ne!(light.panel_fill, dark.panel_fill);
    assert_ne!(light.accent, dark.accent);
}

#[test]
fn dark_palette_keeps_readable_contrast() {
    let light = palette(Theme::Light);
    let dark = palette(Theme::Dark);

    // Dark backgrounds must actually be darker than the light ones.
    assert!(luminance(dark.window_fill) < luminance(light.window_fill));
    assert!(luminance(dark.card_fill) < luminance(light.card_fill));

    // Foreground colours must contrast with the dark card background.
    assert!(luminance(dark.muted_text) > luminance(dark.card_fill));
    assert!(luminance(dark.accent) > luminance(dark.card_fill));
    assert!(luminance(dark.status_running) > luminance(dark.card_fill));
    assert!(luminance(dark.status_error) > luminance(dark.card_fill));
}

fn luminance(color: eframe::egui::Color32) -> u32 {
    u32::from(color.r()) + u32::from(color.g()) + u32::from(color.b())
}
