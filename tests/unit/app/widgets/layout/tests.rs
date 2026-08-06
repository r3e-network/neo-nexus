use super::{tiles_per_row, MIN_TILE_WIDTH};

#[test]
fn a_wide_row_keeps_every_tile_on_one_line() {
    assert_eq!(tiles_per_row(MIN_TILE_WIDTH * 4.0, 4), 4);
    assert_eq!(tiles_per_row(2000.0, 4), 4);
}

#[test]
fn a_narrow_row_balances_instead_of_leaving_an_orphan() {
    // Room for three tiles but four to place: 2 + 2 reads better than 3 + 1.
    assert_eq!(tiles_per_row(MIN_TILE_WIDTH * 3.0, 4), 2);
    // Room for two with five to place: 2 + 2 + 1 is already balanced.
    assert_eq!(tiles_per_row(MIN_TILE_WIDTH * 2.0, 5), 2);
}

#[test]
fn a_tile_is_never_narrower_than_the_minimum_when_the_row_can_avoid_it() {
    for count in 1..=6 {
        for slots in 1..=6 {
            let available = MIN_TILE_WIDTH * slots as f32;
            let per_row = tiles_per_row(available, count);
            assert!(per_row >= 1, "at least one tile per row");
            assert!(per_row <= count, "never more tiles than exist");
            if slots < count {
                assert!(
                    per_row <= slots,
                    "{per_row} tiles cannot fit in {slots} slots",
                );
            }
        }
    }
}

#[test]
fn a_column_too_narrow_for_even_one_tile_still_renders_one() {
    assert_eq!(tiles_per_row(10.0, 4), 1);
    assert_eq!(tiles_per_row(0.0, 1), 1);
}
