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

/// WCAG 2.x relative luminance, linearised per channel. See
/// https://www.w3.org/TR/WCAG21/#dfn-relative-luminance.
fn relative_luminance(color: Color32) -> f32 {
    let channel = |c: u8| {
        let s = c as f32 / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

/// WCAG contrast ratio between a foreground and background colour. 1.0 is no
/// contrast; 21.0 is the maximum (black on white).
fn contrast_ratio(foreground: Color32, background: Color32) -> f32 {
    let fg = relative_luminance(foreground);
    let bg = relative_luminance(background);
    let (lighter, darker) = if fg >= bg { (fg, bg) } else { (bg, fg) };
    (lighter + 0.05) / (darker + 0.05)
}

#[test]
fn primary_body_text_meets_wcag_aa_on_every_surface() {
    for theme in [Theme::Light, Theme::Dark] {
        let palette = palette(theme);
        // AA requires 4.5:1 for normal body text.
        for surface in [palette.card_fill, palette.panel_fill, palette.window_fill] {
            let ratio = contrast_ratio(palette.text, surface);
            assert!(
                ratio >= 4.5,
                "body text {:?} on surface {:?} in {theme:?} is {ratio:.2}:1 (AA needs 4.5)",
                palette.text,
                surface,
            );
        }
    }
}

#[test]
fn accent_labels_and_status_text_meet_wcag_aa_against_their_backgrounds() {
    for theme in [Theme::Light, Theme::Dark] {
        let palette = palette(theme);

        // Labels drawn on the accent fill (primary buttons, selections) use on_accent.
        let on_accent = contrast_ratio(palette.on_accent, palette.accent);
        assert!(
            on_accent >= 4.5,
            "on_accent {:?} on accent {:?} in {theme:?} is {on_accent:.2}:1 (AA needs 4.5)",
            palette.on_accent,
            palette.accent,
        );

        // Status hues are used as inline text on card surfaces (severity labels).
        for (name, status) in [
            ("running", palette.status_running),
            ("starting", palette.status_starting),
            ("error", palette.status_error),
        ] {
            let ratio = contrast_ratio(status, palette.card_fill);
            assert!(
                ratio >= 3.0,
                "{name} status {:?} on card {:?} in {theme:?} is {ratio:.2}:1 (UI components need 3.0)",
                status,
                palette.card_fill,
            );
        }
    }
}

#[test]
fn muted_caption_text_remains_legible_on_cards() {
    for theme in [Theme::Light, Theme::Dark] {
        let palette = palette(theme);
        // Muted captions are small, so they are the hardest case for contrast.
        let ratio = contrast_ratio(palette.muted_text, palette.card_fill);
        assert!(
            ratio >= 4.5,
            "muted_text {:?} on card {:?} in {theme:?} is {ratio:.2}:1 (AA needs 4.5)",
            palette.muted_text,
            palette.card_fill,
        );
    }
}

#[test]
fn dark_palette_keeps_readable_contrast() {
    let light = palette(Theme::Light);
    let dark = palette(Theme::Dark);

    // Dark backgrounds must actually be darker than the light ones.
    assert!(relative_luminance(dark.window_fill) < relative_luminance(light.window_fill));
    assert!(relative_luminance(dark.card_fill) < relative_luminance(light.card_fill));

    // Foreground colours must contrast with the dark card background.
    assert!(relative_luminance(dark.muted_text) > relative_luminance(dark.card_fill));
    assert!(relative_luminance(dark.accent) > relative_luminance(dark.card_fill));
    assert!(relative_luminance(dark.status_running) > relative_luminance(dark.card_fill));
    assert!(relative_luminance(dark.status_error) > relative_luminance(dark.card_fill));
}
