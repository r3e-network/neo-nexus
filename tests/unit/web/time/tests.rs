//! The calendar arithmetic gets checked against dates whose values are
//! independently known, and against the leap-year rollovers where hand-rolled
//! date code usually breaks.

use crate::web::time::{now_unix, relative, time_cell, utc_label};

const SECONDS_PER_DAY: u64 = 86_400;

/// Widely-published anchor values, so a bug in the conversion cannot be
/// cancelled out by a matching bug in the expectation.
#[test]
fn known_unix_seconds_render_their_known_utc_instants() {
    assert_eq!(utc_label(0), "1970-01-01 00:00Z");
    assert_eq!(utc_label(1_000_000_000), "2001-09-09 01:46Z");
    assert_eq!(utc_label(1_234_567_890), "2009-02-13 23:31Z");
    assert_eq!(utc_label(2_000_000_000), "2033-05-18 03:33Z");
    assert_eq!(utc_label(946_684_800), "2000-01-01 00:00Z");
}

#[test]
fn a_leap_day_exists_exactly_where_the_rules_put_it() {
    let new_year_2000 = 946_684_800;
    // 2000 is divisible by 400, so it is a leap year: 28 Feb, 29 Feb, 1 Mar.
    let feb_28 = new_year_2000 + 31 * SECONDS_PER_DAY + 27 * SECONDS_PER_DAY;
    assert_eq!(utc_label(feb_28), "2000-02-28 00:00Z");
    assert_eq!(utc_label(feb_28 + SECONDS_PER_DAY), "2000-02-29 00:00Z");
    assert_eq!(utc_label(feb_28 + 2 * SECONDS_PER_DAY), "2000-03-01 00:00Z");

    // 2001 is not: 28 Feb is followed by 1 Mar.
    let new_year_2001 = new_year_2000 + 366 * SECONDS_PER_DAY;
    let feb_28_2001 = new_year_2001 + 31 * SECONDS_PER_DAY + 27 * SECONDS_PER_DAY;
    assert_eq!(utc_label(feb_28_2001), "2001-02-28 00:00Z");
    assert_eq!(
        utc_label(feb_28_2001 + SECONDS_PER_DAY),
        "2001-03-01 00:00Z",
        "a non-leap year must not invent 29 February"
    );
}

#[test]
fn the_year_rolls_over_at_midnight_utc() {
    let last_instant_of_2009 = 1_234_567_890;
    assert!(utc_label(last_instant_of_2009).starts_with("2009-"));
    let next_year = 1_293_840_000; // 2011-01-01T00:00:00Z
    assert_eq!(utc_label(next_year), "2011-01-01 00:00Z");
}

#[test]
fn labels_are_fixed_width_so_a_column_aligns() {
    for stamp in [0u64, 1, 946_684_800, 2_000_000_000] {
        assert_eq!(
            utc_label(stamp).chars().count(),
            17,
            "unexpected width for {stamp}"
        );
    }
}

#[test]
fn elapsed_time_rounds_to_a_unit_worth_reading() {
    let now = 1_000_000_000;
    assert_eq!(relative(now, now), "just now");
    assert_eq!(relative(now - 30, now), "30s ago");
    assert_eq!(relative(now - 59, now), "59s ago");
    assert_eq!(relative(now - 60, now), "1m ago");
    assert_eq!(relative(now - 7_200, now), "2h ago");
    assert_eq!(relative(now - 3 * 86_400, now), "3d ago");
    // Clock skew must not produce a negative duration or a panic.
    assert_eq!(relative(now + 500, now), "just now");
}

#[test]
fn a_missing_timestamp_reads_as_nothing_rather_than_epoch() {
    assert_eq!(time_cell(None), "—");
}

#[test]
fn a_cell_carries_both_the_reading_and_a_machine_readable_value() {
    let markup = time_cell(Some(1_234_567_890));
    assert!(markup.contains("2009-02-13 23:31Z"), "{markup}");
    assert!(
        markup.contains(r#"datetime="2009-02-13T23:31:30Z""#),
        "the machine-readable form should be the exact instant: {markup}"
    );
    assert!(markup.contains("<time"), "{markup}");
}

#[test]
fn now_is_not_stuck_at_the_epoch() {
    assert!(now_unix() > 1_700_000_000);
}
