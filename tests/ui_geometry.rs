//! Headless geometry verification of the native workbench.
//!
//! egui is an immediate-mode API whose layout is pure logic, so a real frame of
//! `NeoNexusApp` can be rendered against a headless `egui::Context` (the same
//! path egui's own test suite uses via `ctx.run`). Every painted primitive is
//! returned in `full_output.shapes` with its `clip_rect`, and the panel
//! backgrounds are exactly the rectangles egui allocated for each fixed panel.
//! This turns "is the UI professional and well-proportioned?" from a blind,
//! screenshot-only question into a concrete assertion: panel sizes match their
//! design targets, nothing overlaps or collapses, and the fixed workbench fits
//! the 1280x820 design window.

use std::collections::HashSet;

use egui::{Pos2, Rect, Vec2};
use neo_nexus::{repository::Repository, NeoNexusApp};

/// The workbench's design window size (matches `run_native_app`).
const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);

/// The distinct, non-tiny clip rectangles painted during a headless frame.
/// Each fixed panel paints a background fill bounded by its own allocated rect,
/// so the set of large clip rects is the set of laid-out panels.
fn painted_panel_rects() -> Vec<Rect> {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let mut app = NeoNexusApp::new(repository);

    let ctx = egui::Context::default();
    let raw = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..Default::default()
    };
    let output = ctx.run(raw, |ctx| app.render_headless_frame(ctx));

    let mut seen: HashSet<(i32, i32, i32, i32)> = HashSet::new();
    let mut rects = Vec::new();
    for clipped in &output.shapes {
        let rect = clipped.clip_rect;
        // Ignore widget-level slivers; keep only panel-scale regions.
        if rect.width() < 50.0 || rect.height() < 20.0 {
            continue;
        }
        let key = (
            (rect.min.x * 2.0) as i32,
            (rect.min.y * 2.0) as i32,
            (rect.max.x * 2.0) as i32,
            (rect.max.y * 2.0) as i32,
        );
        if seen.insert(key) {
            rects.push(rect);
        }
    }
    rects.sort_by_key(|rect| (rect.min.y as i64 * 10000) + rect.min.x as i64);
    rects
}

/// Returns the panel rect whose width is closest to `target_w` and height
/// closest to `target_h`, by L1 distance in points.
fn panel_near(rects: &[Rect], target_w: f32, target_h: f32) -> Rect {
    *rects
        .iter()
        .min_by(|a, b| {
            let da = (a.width() - target_w).abs() + (a.height() - target_h).abs();
            let db = (b.width() - target_w).abs() + (b.height() - target_h).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("at least one panel rect")
}

#[test]
fn renders_a_full_fixed_panel_set() {
    let rects = painted_panel_rects();
    // Header, status bar, navigation sidebar, inventory + inspector (Summary
    // view shows inventory), and the central workspace: at least five panels.
    assert!(
        rects.len() >= 5,
        "expected a full panel layout, got {} rects: {rects:?}",
        rects.len(),
    );
}

#[test]
fn header_and_status_bar_match_their_design_heights() {
    let rects = painted_panel_rects();
    let header = panel_near(&rects, SCREEN.x, 60.0);
    let status = panel_near(&rects, SCREEN.x, 28.0);
    assert!((header.height() - 60.0).abs() < 1.0, "header {header:?}");
    assert!((status.height() - 28.0).abs() < 1.0, "status {status:?}");
}

#[test]
fn navigation_sidebar_keeps_its_design_width() {
    let rects = painted_panel_rects();
    let sidebar = panel_near(&rects, 212.0, SCREEN.y);
    assert!(
        (sidebar.width() - 212.0).abs() < 1.0,
        "sidebar {sidebar:?} (target 212 wide)",
    );
}

#[test]
fn left_panels_neither_overlap_nor_escape_the_window() {
    let rects = painted_panel_rects();
    let screen = Rect::from_min_size(Pos2::ZERO, SCREEN);
    for rect in &rects {
        assert!(rect.is_finite(), "non-finite rect {rect:?}");
        assert!(
            rect.width() > 0.0 && rect.height() > 0.0,
            "collapsed rect {rect:?}",
        );
        assert!(
            rect.min.x >= -1.0
                && rect.min.y >= -1.0
                && rect.max.x <= screen.max.x + 1.0
                && rect.max.y <= screen.max.y + 1.0,
            "rect {rect:?} escapes window {screen:?}",
        );
    }
    // The two left-side panels (navigation ~212 + inventory ~248) must not
    // stack on top of each other; inventory begins where navigation ends.
    let nav = panel_near(&rects, 212.0, SCREEN.y);
    let inv = panel_near(&rects, 248.0, SCREEN.y);
    assert!(
        inv.min.x >= nav.max.x - 1.0,
        "inventory {inv:?} overlaps navigation {nav:?}",
    );
}

#[test]
fn header_top_status_bottom() {
    let rects = painted_panel_rects();
    let header = panel_near(&rects, SCREEN.x, 60.0);
    let status = panel_near(&rects, SCREEN.x, 28.0);
    assert!(
        (header.min.y - 0.0).abs() < 1.0,
        "header not at top: {header:?}"
    );
    assert!(
        (status.max.y - SCREEN.y).abs() < 1.0,
        "status not at bottom: {status:?}",
    );
}
