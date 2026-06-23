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
