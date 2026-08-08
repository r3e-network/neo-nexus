//! The containment contracts themselves, and the ledger of what still fails.

use super::harness::*;
use super::sub_tabs::SECTIONS;
use super::surfaces::*;

/// Sub-tabs known not to fit yet, and what is cut off on each.
///
/// This list is a **debt ledger, not a permission slip**. Widening the contract
/// from 6 views to every sub-tab found eight; on the pre-v3.3 tree the same sweep
/// reports 38, so they are what is left of a much larger problem rather than
/// something new. Three of the eight have since been fixed — Nodes > Roles, the
/// private-network Plan stage, and the wallet registry — leaving these five.
/// Each one needs the same treatment the contained surfaces already got —
/// paging, sectioning, or shedding content — which is design work per surface,
/// not a layout constant to nudge.
///
/// The test below asserts in **both** directions: no unlisted sub-tab may
/// overflow, and every listed one must still overflow. So fixing a surface
/// fails the suite until its entry is deleted, and the ledger cannot quietly
/// outlive the debt.
const KNOWN_UNCONTAINED: [(&str, &str); 5] = [
    (
        "settings/alerts",
        "the webhook form overflows both axes: ~167pt past the bottom and ~86pt past the right",
    ),
    (
        "settings/upgrades",
        "the upgrade-policy form runs ~93pt past the bottom",
    ),
    (
        "monitor/telemetry",
        "the telemetry table renders a fixed row count instead of a height-derived one (~93pt)",
    ),
    (
        "runtimes/sync",
        "the applied-version list renders a fixed row count instead of a height-derived one (~78pt)",
    ),
    (
        "federation/editor",
        "the remote-profile editor runs ~39pt past the bottom",
    ),
];

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
        KNOWN_UNCONTAINED.len() <= 5,
        "a new sub-tab was added to KNOWN_UNCONTAINED; fix the surface instead",
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
