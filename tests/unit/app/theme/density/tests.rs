use super::{DensityMetrics, UiDensity};

#[test]
fn comfortable_matches_shipped_style_baseline() {
    let m = DensityMetrics::COMFORTABLE;
    assert_eq!(m.interact_y, 28.0);
    assert_eq!(m.button_pad_x, 14.0);
    assert_eq!(m.button_pad_y, 8.0);
    assert_eq!(m.item_spacing_x, 10.0);
    assert_eq!(m.item_spacing_y, 8.0);
    assert_eq!(m.nav_row_height, 34.0);
    assert_eq!(m.list_row_compact, 44.0);
    assert_eq!(m.list_row_expanded, 56.0);
    assert_eq!(m.journal_slot, 52.0);
}

#[test]
fn compact_densifies_controls_only() {
    let c = DensityMetrics::COMPACT;
    let k = DensityMetrics::COMFORTABLE;
    assert!(c.interact_y < k.interact_y);
    assert!(c.button_pad_x < k.button_pad_x);
    assert!(c.button_pad_y < k.button_pad_y);
    assert!(c.item_spacing_x < k.item_spacing_x);
    assert!(c.item_spacing_y < k.item_spacing_y);
    assert!(c.nav_row_height < k.nav_row_height);
    // List heights stay Comfortable until geometry proof.
    assert_eq!(c.list_row_compact, k.list_row_compact);
    assert_eq!(c.list_row_expanded, k.list_row_expanded);
    assert_eq!(c.journal_slot, k.journal_slot);
}

#[test]
fn for_density_selects_table() {
    assert_eq!(
        DensityMetrics::for_density(UiDensity::Comfortable),
        DensityMetrics::COMFORTABLE
    );
    assert_eq!(
        DensityMetrics::for_density(UiDensity::Compact),
        DensityMetrics::COMPACT
    );
}

#[test]
fn density_persist_keys_round_trip() {
    for density in [UiDensity::Comfortable, UiDensity::Compact] {
        assert_eq!(
            UiDensity::from_persist_key(density.persist_key()),
            Some(density)
        );
    }
    assert_eq!(UiDensity::from_persist_key("unknown"), None);
}

#[test]
fn default_density_is_comfortable() {
    assert_eq!(UiDensity::default(), UiDensity::Comfortable);
}

/// Geometry proof gate (PR-14-full): Compact list anatomy stays locked to
/// Comfortable. Future Compact list densification may only change these
/// three fields after a new proof PR rewrites this assertion with evidence.
#[test]
fn compact_list_anatomy_gate_locked_to_comfortable() {
    let c = DensityMetrics::COMPACT;
    let k = DensityMetrics::COMFORTABLE;
    assert_eq!(
        (c.list_row_compact, c.list_row_expanded, c.journal_slot),
        (k.list_row_compact, k.list_row_expanded, k.journal_slot),
        "Compact list heights may only change after a geometry proof PR"
    );
    // Document the shipped Comfortable anatomy so a silent drift fails loudly.
    assert_eq!(k.list_row_compact, 44.0);
    assert_eq!(k.list_row_expanded, 56.0);
    assert_eq!(k.journal_slot, 52.0);
}
