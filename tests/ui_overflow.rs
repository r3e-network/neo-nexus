//! Headless containment verification: does the workbench paint inside itself?
//!
//! `tests/ui_geometry.rs` measures each panel's **clip rect**, which is the
//! region egui allows a panel to draw in. That is not the same as the region a
//! panel's content actually asks for, and the difference hid three real layout
//! faults at once: inventory rows grew wider with every item until the list
//! shoved the central workspace into a 271pt slot; the status bar laid out 38pt
//! of content inside a fixed 28pt strip; and the inspector stacked roughly
//! 1300pt of facts into a 725pt column so its lifecycle buttons were painted
//! where no one could see them.
//!
//! Every one of those passed the clip-rect contract. This one renders real
//! frames and asserts that painted geometry stays inside the region that will
//! actually be shown, so a container that silently truncates its own content
//! fails the build instead of shipping.

use egui::{Pos2, Rect, Shape, Vec2};
use neo_nexus::{repository::Repository, NeoNexusApp};

/// The workbench's design window size (matches `run_native_app`).
const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);

/// Slack for hairline strokes and panel resize handles, which legitimately
/// paint a pixel or two outside their content region.
const TOLERANCE: f32 = 3.0;

/// A painted rectangle that extends past the clip region it belongs to.
struct Overflow {
    axis: &'static str,
    amount: f32,
    clip: Rect,
    painted: Rect,
}

fn overflows(view_key: &str, dark: bool) -> Vec<Overflow> {
    overflows_with(view_key, dark, true)
}

fn overflows_with(view_key: &str, dark: bool, inspector: bool) -> Vec<Overflow> {
    overflows_in(view_key, dark, inspector, &[])
}

fn overflows_in(
    view_key: &str,
    dark: bool,
    inspector: bool,
    sections: &[(&str, &str)],
) -> Vec<Overflow> {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&path).unwrap();
    repository.save_app_dark_mode(dark).unwrap();
    repository.save_app_inspector_visible(inspector).unwrap();
    repository.save_workspace_last_view(view_key).unwrap();
    for (setting_key, section_key) in sections {
        repository
            .save_workspace_section(setting_key, section_key)
            .unwrap();
    }
    drop(repository);
    let mut app = NeoNexusApp::new(Repository::open(&path).unwrap());

    let context = egui::Context::default();
    let raw = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..Default::default()
    };
    // Two frames: the first settles panel widths and the font atlas, the second
    // is the steady state an operator would see.
    let _ = context.run(raw.clone(), |ctx| app.render_headless_frame(ctx));
    let output = context.run(raw, |ctx| app.render_headless_frame(ctx));

    let mut found = Vec::new();
    for clipped in &output.shapes {
        let Shape::Rect(rect) = &clipped.shape else {
            continue;
        };
        // Ignore slivers: only container-scale surfaces matter here.
        if rect.rect.width() < 40.0 || rect.rect.height() < 12.0 {
            continue;
        }
        let clip = clipped.clip_rect;
        for (axis, amount) in [
            ("right", rect.rect.max.x - clip.max.x),
            ("left", clip.min.x - rect.rect.min.x),
            ("bottom", rect.rect.max.y - clip.max.y),
        ] {
            if amount > TOLERANCE {
                found.push(Overflow {
                    axis,
                    amount,
                    clip,
                    painted: rect.rect,
                });
            }
        }
    }
    found
}

fn report(view: &str, found: &[Overflow]) -> String {
    let mut worst: Vec<&Overflow> = found.iter().collect();
    worst.sort_by(|a, b| b.amount.total_cmp(&a.amount));
    worst
        .iter()
        .take(3)
        .map(|overflow| {
            let (painted, clip) = if overflow.axis == "bottom" {
                (
                    (overflow.painted.min.y, overflow.painted.max.y),
                    (overflow.clip.min.y, overflow.clip.max.y),
                )
            } else {
                (
                    (overflow.painted.min.x, overflow.painted.max.x),
                    (overflow.clip.min.x, overflow.clip.max.x),
                )
            };
            format!(
                "\n  {view}: {:.0}pt past the {} edge — painted {:.0}..{:.0}, clip {:.0}..{:.0}",
                overflow.amount, overflow.axis, painted.0, painted.1, clip.0, clip.1,
            )
        })
        .collect()
}

/// Every primary destination, with the inspector column both open and closed —
/// the two layouts an operator actually switches between.
const VIEWS: [&str; 6] = [
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

/// Every sub-tab of every view, as (label, view, persistence key, section key).
///
/// The two contracts above iterate `VIEWS`, which lands each view on whichever
/// sub-tab happened to be persisted — so a surface reachable only by clicking a
/// segment was never measured, and two of them were shipping with their primary
/// action laid out below the panel. A view is not one surface; it is as many
/// surfaces as it has segments, and each one has to be contained on its own.
const SECTIONS: [(&str, &str, &str, &str); 32] = [
    ("nodes/studio", "nodes", "workspace.section.nodes", "studio"),
    ("nodes/config", "nodes", "workspace.section.nodes", "config"),
    ("nodes/roles", "nodes", "workspace.section.nodes", "roles"),
    (
        "nodes/plugins",
        "nodes",
        "workspace.section.nodes",
        "plugins",
    ),
    ("nodes/logs", "nodes", "workspace.section.nodes", "logs"),
    ("nodes/health", "nodes", "workspace.section.nodes", "health"),
    (
        "roles/presets",
        "roles",
        "workspace.section.roles",
        "presets",
    ),
    ("roles/plan", "roles", "workspace.section.roles", "plan"),
    (
        "operations/readiness",
        "operations",
        "workspace.section.operations",
        "readiness",
    ),
    (
        "operations/action-queue",
        "operations",
        "workspace.section.operations",
        "action_queue",
    ),
    (
        "operations/ports",
        "operations",
        "workspace.section.operations",
        "ports",
    ),
    (
        "operations/safety",
        "operations",
        "workspace.section.operations",
        "safety",
    ),
    (
        "operations/journal",
        "operations",
        "workspace.section.operations",
        "journal",
    ),
    (
        "settings/watchdog",
        "settings",
        "workspace.section.settings",
        "watchdog",
    ),
    (
        "settings/upgrades",
        "settings",
        "workspace.section.settings",
        "upgrades",
    ),
    (
        "settings/monitors",
        "settings",
        "workspace.section.settings",
        "monitors",
    ),
    (
        "settings/alerts",
        "settings",
        "workspace.section.settings",
        "alerts",
    ),
    (
        "settings/storage",
        "settings",
        "workspace.section.settings",
        "storage",
    ),
    (
        "settings/release",
        "settings",
        "workspace.section.settings",
        "release",
    ),
    (
        "runtimes/install",
        "runtimes",
        "workspace.section.runtimes",
        "install",
    ),
    (
        "runtimes/catalog",
        "runtimes",
        "workspace.section.runtimes",
        "catalog",
    ),
    (
        "runtimes/installed",
        "runtimes",
        "workspace.section.runtimes",
        "installed",
    ),
    (
        "runtimes/applied",
        "runtimes",
        "workspace.section.runtimes",
        "applied",
    ),
    (
        "runtimes/sync",
        "runtimes",
        "workspace.section.runtimes",
        "sync",
    ),
    (
        "monitor/pressure",
        "monitor",
        "workspace.section.monitor",
        "pressure",
    ),
    (
        "monitor/telemetry",
        "monitor",
        "workspace.section.monitor",
        "telemetry",
    ),
    (
        "monitor/processes",
        "monitor",
        "workspace.section.monitor",
        "processes",
    ),
    (
        "monitor/chain-duties",
        "monitor",
        "workspace.section.monitor",
        "chain-duties",
    ),
    (
        "federation/profiles",
        "federation",
        "workspace.section.federation",
        "profiles",
    ),
    (
        "federation/editor",
        "federation",
        "workspace.section.federation",
        "editor",
    ),
    (
        "federation/inspector",
        "federation",
        "workspace.section.federation",
        "inspector",
    ),
    (
        "federation/governance",
        "federation",
        "workspace.section.federation",
        "governance",
    ),
];

/// Sub-tabs known not to fit yet, and what is cut off on each.
///
/// This list is a **debt ledger, not a permission slip**. Widening the contract
/// from 6 views to 32 sub-tabs found these; on `main` the same sweep reports 38,
/// so they are what is left of a much larger problem rather than something new.
/// Each one needs the same treatment the contained surfaces already got —
/// paging, sectioning, or shedding content — which is design work per surface,
/// not a layout constant to nudge.
///
/// The test below asserts in **both** directions: no unlisted sub-tab may
/// overflow, and every listed one must still overflow. So fixing a surface
/// fails the suite until its entry is deleted, and the ledger cannot quietly
/// outlive the debt.
const KNOWN_UNCONTAINED: [(&str, &str); 8] = [
    (
        "roles/presets",
        "Apply Role and the Notary/Observer presets lay out ~414pt below the panel",
    ),
    (
        "roles/plan",
        "the plugin-change list runs ~414pt past the bottom with no paging",
    ),
    (
        "nodes/roles",
        "the per-node duty picker runs ~354pt past the bottom",
    ),
    (
        "settings/alerts",
        "the webhook form overflows both the right edge (~86pt) and the bottom (~167pt)",
    ),
    (
        "settings/upgrades",
        "the upgrade-policy form runs ~93pt past the bottom",
    ),
    (
        "monitor/telemetry",
        "the telemetry table runs ~93pt past the bottom with no paging",
    ),
    (
        "runtimes/sync",
        "the applied-version list runs ~78pt past the bottom",
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

    for (label, view, key, section) in SECTIONS {
        let mut overflowed = false;
        for inspector in [true, false] {
            let found = overflows_in(view, false, inspector, &[(key, section)]);
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
        KNOWN_UNCONTAINED.len() <= 8,
        "a new sub-tab was added to KNOWN_UNCONTAINED; fix the surface instead",
    );
    for (label, reason) in KNOWN_UNCONTAINED {
        assert!(
            SECTIONS.iter().any(|(name, ..)| *name == label),
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
