use super::glyph;
use crate::app::view::View;

/// Every workspace view must resolve to a real Phosphor glyph, otherwise the
/// sidebar would render a blank pictogram. Phosphor code points live in the
/// supplementary private-use area (U+E000..U+F8FF), so a single non-empty
/// character in that range confirms the mapping is wired to a real icon.
#[test]
fn every_view_maps_to_a_phosphor_glyph() {
    for view in View::ALL {
        let icon = glyph(view);
        assert_eq!(
            icon.chars().count(),
            1,
            "view {view:?} must map to exactly one icon character",
        );
        let code = icon.chars().next().unwrap() as u32;
        assert!(
            (0xE000..=0xF8FF).contains(&code),
            "view {view:?} glyph U+{code:04X} is outside the Phosphor private-use range",
        );
    }
}

/// Distinct workspace pages should not all share one pictogram; that would make
/// the sidebar harder, not easier, to scan. Summary and Settings are allowed to
/// be the bookends, but the set as a whole must use more than a couple of icons.
#[test]
fn icons_are_varied_across_views() {
    let unique: std::collections::BTreeSet<_> = View::ALL.iter().map(|view| glyph(*view)).collect();
    assert!(
        unique.len() >= View::ALL.len() / 2,
        "expected at least half of the views to use distinct icons, got {} unique of {}",
        unique.len(),
        View::ALL.len(),
    );
}

/// The icon font must be installed into *every* context, not just the first one
/// the process ever creates. A process-wide install guard left every later
/// context without the Phosphor family, so all its glyphs rendered as tofu
/// boxes — invisible in the running app, but immediately visible to a second
/// headless render.
/// `set_fonts` lands on the following frame, so two frames is the earliest a
/// context can have the family — but a context that never gets it (the bug)
/// stays at zero no matter how many frames run.
#[test]
fn every_context_receives_the_icon_font() {
    for context_index in 0..3 {
        assert_eq!(
            phosphor_family_entries(2),
            1,
            "context {context_index} is missing the Phosphor icon font",
        );
    }
}

/// Installing on every frame must be idempotent rather than appending a second
/// family entry each time.
#[test]
fn installing_every_frame_does_not_duplicate_the_family_entry() {
    assert_eq!(phosphor_family_entries(3), 1);
}

/// Runs `install` on a fresh context for `frames` frames and returns how many
/// times the Phosphor key appears in the proportional family.
fn phosphor_family_entries(frames: usize) -> usize {
    use eframe::egui;
    let context = egui::Context::default();
    let mut entries = 0;
    for _ in 0..frames {
        let _ = context.run(egui::RawInput::default(), |ctx| {
            super::install(ctx);
            entries = ctx.fonts(|fonts| {
                fonts
                    .definitions()
                    .families
                    .get(&egui::FontFamily::Proportional)
                    .map(|keys| {
                        keys.iter()
                            .filter(|key| key.as_str() == super::PHOSPHOR_FONT_KEY)
                            .count()
                    })
                    .unwrap_or_default()
            });
        });
    }
    entries
}
