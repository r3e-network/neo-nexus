//! Page-fit / geometry gates for the v3.2.0 UI density metrics.
//!
//! These exercise the pure [`DensityMode`] API — no server, no database. They
//! pin the pixel steps the stylesheet and the compact fleet layout depend on,
//! so a change to a row height or a spacing token cannot slip through silently.
//!
//! The repo's other in-crate unit tests are `#[path]`-included from `src/` and
//! reach the type through `crate::…`. `DensityMode` and its `spacing` helpers
//! are re-exported publicly via `neo_nexus::web::assets`, so these gates run as
//! an ordinary external test crate against that public surface instead —
//! keeping the density test suite entirely additive to `src/`.

use neo_nexus::web::assets::{spacing, DensityMode};

#[test]
fn row_height_matches_the_documented_slot_per_mode() {
    assert_eq!(
        DensityMode::Comfortable.row_height(),
        56,
        "comfortable rows are 56px"
    );
    assert_eq!(
        DensityMode::Compact.row_height(),
        40,
        "compact rows tighten to 40px"
    );
}

#[test]
fn margin_tokens_match_the_documented_steps_per_mode() {
    assert_eq!(DensityMode::Comfortable.margin_xs(), 8);
    assert_eq!(DensityMode::Comfortable.margin_sm(), 12);
    assert_eq!(DensityMode::Compact.margin_xs(), 4);
    assert_eq!(DensityMode::Compact.margin_sm(), 8);
}

#[test]
fn from_str_maps_compact_and_defaults_everything_else_to_comfortable() {
    assert_eq!(DensityMode::from_str("compact"), DensityMode::Compact);
    // Case-insensitive: the persisted key is lower-case, but a hand-edited
    // value must still resolve.
    assert_eq!(DensityMode::from_str("Compact"), DensityMode::Compact);
    assert_eq!(DensityMode::from_str("COMPACT"), DensityMode::Compact);

    assert_eq!(
        DensityMode::from_str("comfortable"),
        DensityMode::Comfortable
    );
    // Anything unrecognised — an empty string, a typo, a stale key — falls back
    // to the comfortable default rather than erroring.
    assert_eq!(DensityMode::from_str(""), DensityMode::Comfortable);
    assert_eq!(DensityMode::from_str("cozy"), DensityMode::Comfortable);
    assert_eq!(
        DensityMode::from_str("compactish"),
        DensityMode::Comfortable
    );
}

#[test]
fn as_str_and_from_str_round_trip_through_the_persisted_key() {
    for mode in [DensityMode::Comfortable, DensityMode::Compact] {
        assert_eq!(
            DensityMode::from_str(mode.as_str()),
            mode,
            "{mode:?} must survive save/load through as_str/from_str"
        );
    }
    assert_eq!(DensityMode::Comfortable.as_str(), "comfortable");
    assert_eq!(DensityMode::Compact.as_str(), "compact");
}

#[test]
fn body_class_is_the_shell_modifier_the_stylesheet_scopes() {
    assert_eq!(DensityMode::Comfortable.body_class(), "density-comfortable");
    assert_eq!(DensityMode::Compact.body_class(), "density-compact");
}

#[test]
fn default_is_comfortable() {
    assert_eq!(DensityMode::DEFAULT, DensityMode::Comfortable);
    assert_eq!(DensityMode::default(), DensityMode::Comfortable);
}

#[test]
fn spacing_tokens_track_the_mode_margins() {
    for mode in [DensityMode::Comfortable, DensityMode::Compact] {
        assert_eq!(
            spacing::xs(mode),
            mode.margin_xs(),
            "spacing::xs must match margin_xs for {mode:?}"
        );
        assert_eq!(
            spacing::sm(mode),
            mode.margin_sm(),
            "spacing::sm must match margin_sm for {mode:?}"
        );
    }
}
