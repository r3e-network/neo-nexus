use super::{fits_side_by_side, fits_side_by_side_at, MIN_COLUMN_WIDTH};
use crate::app::theme;

#[test]
fn two_columns_need_both_minimums_plus_the_gap() {
    let exact = MIN_COLUMN_WIDTH * 2.0 + theme::SM;
    assert!(fits_side_by_side(exact));
    assert!(!fits_side_by_side(exact - 1.0));
}

#[test]
fn the_default_minimum_rejects_the_narrow_central_workspace() {
    // With the sidebar (212), inventory (~252) and inspector (~304) open, the
    // central workspace is around 450pt. That must never split in two.
    assert!(!fits_side_by_side(450.0));
    // With the inspector closed there is room.
    assert!(fits_side_by_side(760.0));
}

#[test]
fn a_denser_minimum_allows_two_up_sooner() {
    assert!(!fits_side_by_side_at(500.0, MIN_COLUMN_WIDTH));
    assert!(fits_side_by_side_at(500.0, 220.0));
}

#[test]
fn a_zero_or_negative_width_never_fits_two_columns() {
    assert!(!fits_side_by_side(0.0));
    assert!(!fits_side_by_side(-100.0));
}
