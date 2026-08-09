//! The containment contracts themselves, and the ledger of what still fails.

use super::harness::*;
use super::sub_tabs::SECTIONS;
use super::surfaces::*;

/// Sub-tabs allowed not to fit. **Empty, and it stays that way.**
///
/// Widening the contract from 6 views to all 32 sub-tabs found eight surfaces
/// laying out content below a panel that does not scroll; the same sweep against
/// the pre-v3.3 tree reports 38. All of them are fixed, so this list is now a
/// tripwire rather than a ledger: adding an entry means shipping a surface an
/// operator cannot fully see, and `the_uncontained_ledger_does_not_grow` fails
/// the build for it.
const KNOWN_UNCONTAINED: [(&str, &str); 0] = [];

fn known_uncontained(label: &str) -> bool {
    KNOWN_UNCONTAINED.iter().any(|(known, _)| *known == label)
}

/// Every segment is measured on both axes, rather than only whichever segment
/// happened to be the persisted default.
///
/// The vertical axis is the one that hides things outright: egui culls a widget
/// laid out entirely below its panel, so a button down there is not merely
/// awkward to reach — it does not exist for the operator. That is how the Roles
/// tab shipped with its only action unreachable.
#[test]
fn every_sub_tab_is_contained_or_a_declared_exception() {
    let mut failures = String::new();
    let mut fixed = Vec::new();

    let flat = SECTIONS
        .into_iter()
        .map(|(label, view, key, section)| (label, view, vec![(key, section)]));
    let nested = NESTED
        .into_iter()
        .map(|(label, view, keys)| (label, view, keys.to_vec()));
    for (label, view, keys) in flat.chain(nested) {
        let mut overflowed = false;
        for inspector in [true, false] {
            let found = overflows_in(view, false, inspector, &keys);
            if found.is_empty() {
                continue;
            }
            overflowed = true;
            if !known_uncontained(label) {
                failures.push_str(&report(label, &found));
            }
        }
        if !overflowed && known_uncontained(label) {
            fixed.push(label);
        }
    }

    assert!(
        failures.is_empty(),
        "sub-tabs lay out content outside the panel that holds it, and the \
         workbench does not scroll, so that content can never be seen or \
         clicked:{failures}",
    );
    assert!(
        fixed.is_empty(),
        "these sub-tabs are contained now — delete them from \
         KNOWN_UNCONTAINED so the ledger keeps shrinking: {fixed:?}",
    );
}

/// The ledger may only ever shrink.
#[test]
fn the_uncontained_ledger_does_not_grow() {
    assert!(
        KNOWN_UNCONTAINED.is_empty(),
        "every sub-tab fits today. Adding an entry here ships a surface whose \
         content is laid out where egui will cull it — fix the surface instead.",
    );
    for (label, reason) in KNOWN_UNCONTAINED {
        assert!(
            SECTIONS.iter().any(|(name, ..)| *name == label)
                || NESTED.iter().any(|(name, ..)| *name == label),
            "{label} is not a real sub-tab",
        );
        assert!(!reason.is_empty(), "{label} needs a reason");
    }
}

#[test]
fn the_dark_theme_lays_out_identically_to_the_light_theme() {
    let light = overflows("summary", false).len();
    let dark = overflows("summary", true).len();
    assert_eq!(
        light, dark,
        "theme changed layout containment: {light} light overflows vs {dark} dark",
    );
}

/// the two layouts an operator actually switches between.
pub(super) const VIEWS: [&str; 6] = [
    "summary",
    "nodes",
    "runtimes",
    "federation",
    "operations",
    "settings",
];

#[test]
fn no_surface_paints_outside_its_own_column() {
    let mut failures = String::new();
    for view in VIEWS {
        for inspector in [true, false] {
            let found: Vec<Overflow> = overflows_with(view, false, inspector)
                .into_iter()
                .filter(|overflow| overflow.axis != "bottom")
                .collect();
            failures.push_str(&report(view, &found));
        }
    }
    assert!(
        failures.is_empty(),
        "surfaces paint outside the column that will clip them, so their \
         content is silently truncated:{failures}",
    );
}

/// The workbench does not scroll. Anything laid out below the bottom of its
/// panel is not reachable by any means: egui culls the widgets that fall
/// entirely outside, so the content is silently dropped rather than shown.
/// A surface that does not fit must page, section, or shed content — never
/// simply run off the end.
#[test]
fn no_surface_paints_below_the_panel_that_holds_it() {
    let mut failures = String::new();
    for view in VIEWS {
        for inspector in [true, false] {
            let found: Vec<Overflow> = overflows_with(view, false, inspector)
                .into_iter()
                .filter(|overflow| overflow.axis == "bottom")
                .collect();
            failures.push_str(&report(view, &found));
        }
    }
    assert!(
        failures.is_empty(),
        "surfaces lay out content below the bottom of a panel that does not \
         scroll, so that content can never be seen or clicked:{failures}",
    );
}
