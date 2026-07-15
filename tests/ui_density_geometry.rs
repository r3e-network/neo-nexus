//! Headless Compact density geometry proof (v3.1 PR-14-full).
//!
//! Compact must densify **controls** (interact size, button pad, spacing)
//! while chrome panel sizes (header 60 / status 28 / sidebar 212) and
//! list-row metrics stay identical to Comfortable. This gate is the only
//! path allowed to unlock Compact list-height changes later — until then
//! Compact list heights equal Comfortable.

use std::collections::HashSet;

use egui::{Pos2, Rect, Vec2};
use neo_nexus::{repository::Repository, NeoNexusApp};

const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);

const CHROME_HEADER: f32 = 60.0;
const CHROME_STATUS: f32 = 28.0;
const CHROME_SIDEBAR: f32 = 212.0;

struct DensityFrame {
    rects: Vec<Rect>,
    interact_y: f32,
    button_pad: egui::Vec2,
    item_spacing: egui::Vec2,
}

fn paint_with_density(density_key: &str) -> DensityFrame {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    repository
        .save_app_ui_density(density_key)
        .expect("persist density for headless proof");
    let mut app = NeoNexusApp::new(repository);

    let ctx = egui::Context::default();
    let raw = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..Default::default()
    };

    let mut interact_y = 0.0;
    let mut button_pad = egui::Vec2::ZERO;
    let mut item_spacing = egui::Vec2::ZERO;
    let output = ctx.run(raw, |ctx| {
        app.render_headless_frame(ctx);
        let spacing = &ctx.style().spacing;
        interact_y = spacing.interact_size.y;
        button_pad = spacing.button_padding;
        item_spacing = spacing.item_spacing;
    });

    let mut seen: HashSet<(i32, i32, i32, i32)> = HashSet::new();
    let mut rects = Vec::new();
    for clipped in &output.shapes {
        let rect = clipped.clip_rect;
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

    DensityFrame {
        rects,
        interact_y,
        button_pad,
        item_spacing,
    }
}

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

fn assert_chrome_invariant(frame: &DensityFrame, density: &str) {
    let header = panel_near(&frame.rects, SCREEN.x, CHROME_HEADER);
    let status = panel_near(&frame.rects, SCREEN.x, CHROME_STATUS);
    let sidebar = panel_near(&frame.rects, CHROME_SIDEBAR, SCREEN.y);
    assert!(
        (header.height() - CHROME_HEADER).abs() < 1.0,
        "{density}: header {header:?}"
    );
    assert!(
        (status.height() - CHROME_STATUS).abs() < 1.0,
        "{density}: status {status:?}"
    );
    assert!(
        (sidebar.width() - CHROME_SIDEBAR).abs() < 1.0,
        "{density}: sidebar {sidebar:?}"
    );
}

#[test]
fn comfortable_control_metrics_match_baseline() {
    let frame = paint_with_density("comfortable");
    assert_chrome_invariant(&frame, "comfortable");
    assert!(
        (frame.interact_y - 28.0).abs() < 0.5,
        "comfortable interact_y {}",
        frame.interact_y
    );
    assert!(
        (frame.button_pad.x - 14.0).abs() < 0.5 && (frame.button_pad.y - 8.0).abs() < 0.5,
        "comfortable button_pad {:?}",
        frame.button_pad
    );
    assert!(
        (frame.item_spacing.x - 10.0).abs() < 0.5 && (frame.item_spacing.y - 8.0).abs() < 0.5,
        "comfortable item_spacing {:?}",
        frame.item_spacing
    );
}

#[test]
fn compact_densifies_controls_and_keeps_chrome() {
    let frame = paint_with_density("compact");
    assert_chrome_invariant(&frame, "compact");
    // Control metrics densify.
    assert!(
        (frame.interact_y - 24.0).abs() < 0.5,
        "compact interact_y {}",
        frame.interact_y
    );
    assert!(
        (frame.button_pad.x - 10.0).abs() < 0.5 && (frame.button_pad.y - 6.0).abs() < 0.5,
        "compact button_pad {:?}",
        frame.button_pad
    );
    assert!(
        (frame.item_spacing.x - 8.0).abs() < 0.5 && (frame.item_spacing.y - 6.0).abs() < 0.5,
        "compact item_spacing {:?}",
        frame.item_spacing
    );
}

#[test]
fn compact_and_comfortable_share_chrome_geometry() {
    let comfortable = paint_with_density("comfortable");
    let compact = paint_with_density("compact");
    assert_chrome_invariant(&comfortable, "comfortable");
    assert_chrome_invariant(&compact, "compact");

    let c_header = panel_near(&comfortable.rects, SCREEN.x, CHROME_HEADER);
    let k_header = panel_near(&compact.rects, SCREEN.x, CHROME_HEADER);
    assert!(
        (c_header.height() - k_header.height()).abs() < 1.0,
        "header height must be density-invariant: comfortable {c_header:?} compact {k_header:?}"
    );

    let c_side = panel_near(&comfortable.rects, CHROME_SIDEBAR, SCREEN.y);
    let k_side = panel_near(&compact.rects, CHROME_SIDEBAR, SCREEN.y);
    assert!(
        (c_side.width() - k_side.width()).abs() < 1.0,
        "sidebar width must be density-invariant: comfortable {c_side:?} compact {k_side:?}"
    );

    // Controls must actually differ so Compact is never a no-op.
    assert!(
        compact.interact_y < comfortable.interact_y,
        "compact must densify interact_y ({} vs {})",
        compact.interact_y,
        comfortable.interact_y
    );
}
