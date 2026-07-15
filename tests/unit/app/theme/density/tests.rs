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
fn compact_densifies_controls_and_single_line_lists() {
    let c = DensityMetrics::COMPACT;
    let k = DensityMetrics::COMFORTABLE;
    assert!(c.interact_y < k.interact_y);
    assert!(c.button_pad_x < k.button_pad_x);
    assert!(c.button_pad_y < k.button_pad_y);
    assert!(c.item_spacing_x < k.item_spacing_x);
    assert!(c.item_spacing_y < k.item_spacing_y);
    assert!(c.nav_row_height < k.nav_row_height);
    // Single-line inventory/fleet anatomy after geometry proof.
    assert_eq!(c.list_row_compact, 40.0);
    assert_eq!(c.list_row_expanded, 40.0);
    assert!(c.list_row_compact < k.list_row_compact);
    assert!(c.list_row_expanded < k.list_row_expanded);
    // Journal multi-line event rows stay Comfortable-height.
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

/// Geometry proof: Compact inventory page of 7×40 + gaps fits the workbench
/// content column (design math in ui-system-redesign-v3.1.md §1.6).
#[test]
fn compact_inventory_page_fits_workbench_content_column() {
    const NODE_PAGE_SIZE: usize = 7;
    const WORKBENCH_H: f32 = 820.0 - 60.0 - 28.0; // screen − header − status
    // Inventory: filter + mini-stats + pagination leave ~620 usable; use 500 as
    // a conservative lower bound for the list region alone.
    const LIST_REGION_MIN: f32 = 500.0;
    let row = DensityMetrics::COMPACT.list_row_compact;
    let gap = 4.0; // theme::XS
    let page_h = NODE_PAGE_SIZE as f32 * (row + gap);
    assert!(
        page_h < LIST_REGION_MIN,
        "7 compact rows need {page_h}pt; list region ≥ {LIST_REGION_MIN}"
    );
    assert!(
        page_h < WORKBENCH_H,
        "compact inventory page must fit workbench height {WORKBENCH_H}"
    );
    // Single-line anatomy requires ≥ ~40pt (design floor).
    assert!(row >= 40.0);
}
