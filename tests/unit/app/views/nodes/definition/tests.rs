use super::{FIELD_COLUMN_MIN_WIDTH, GROUPS};
use crate::app::widgets::columns_that_fit;

fn groups_per_row(available: f32, count: usize) -> usize {
    columns_that_fit(available, FIELD_COLUMN_MIN_WIDTH, count)
}

#[test]
fn a_wide_pane_shows_every_group_on_one_row() {
    assert_eq!(
        groups_per_row(FIELD_COLUMN_MIN_WIDTH * 3.0, GROUPS.len()),
        GROUPS.len()
    );
}

#[test]
fn a_narrow_pane_wraps_rather_than_shrinking_a_field_below_its_label() {
    assert_eq!(
        groups_per_row(FIELD_COLUMN_MIN_WIDTH * 2.0, GROUPS.len()),
        2
    );
    assert_eq!(groups_per_row(FIELD_COLUMN_MIN_WIDTH, GROUPS.len()), 1);
    assert_eq!(groups_per_row(10.0, GROUPS.len()), 1);
}

#[test]
fn the_column_count_never_exceeds_the_number_of_groups() {
    assert_eq!(groups_per_row(4000.0, GROUPS.len()), GROUPS.len());
    assert_eq!(groups_per_row(4000.0, 1), 1);
}

/// The three groups exist so no single column carries every field: eleven
/// fields in one column paints past the bottom of a non-scrolling panel.
#[test]
fn the_groups_are_named_and_distinct() {
    let mut titles: Vec<&str> = GROUPS.iter().map(|(title, _)| *title).collect();
    assert_eq!(titles, ["Identity", "Runtime", "Ports"]);
    titles.sort_unstable();
    titles.dedup();
    assert_eq!(titles.len(), GROUPS.len());
}
