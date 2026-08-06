//! Pixel-truth verification for the rendered UI.
//!
//! Screen capture (`screencapture`) is NOT a reliable way to verify this app's
//! rendering: on a Retina / scaled macOS desktop it captures the whole virtual
//! workspace (often larger than the screen), the app window may only occupy a
//! corner, and the rest is dark desktop — so a "screenshot" mostly shows empty
//! black space, not the app. Automated vision tools also fabricate precise
//! measurements (e.g. claiming "75% magenta" when pixel counting shows 0%).
//!
//! This test instead renders real 1280x820 frames headlessly (the same path the
//! contract tests use) and rasterises the tessellated triangle meshes — the
//! exact geometry a GPU backend would draw — sampling egui's own font atlas so
//! glyphs, hairlines, and rounded corners come out as real pixels. The PNGs are
//! therefore the OBJECTIVE truth of what the app paints, with zero desktop /
//! window-position / screen-capture interference.
//!
//! It is `#[ignore]`d so it never runs in CI; run it on demand:
//!
//!     cargo test --release --test ui_visual_truth -- --nocapture --ignored
//!
//! Then inspect the PNGs written to /tmp/neonexus_truth_*.png.

#[path = "ui_visual_truth/fixture.rs"]
mod fixture;
#[path = "ui_visual_truth/raster.rs"]
mod raster;

use egui::{Color32, Pos2, Rect, Vec2};
use neo_nexus::NeoNexusApp;

/// Design window size, matching the geometry contract (tests/ui_geometry.rs).
const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);

/// The workbench surfaces worth eyeballing: every primary navigation
/// destination, rendered from a seeded fleet so no page is an empty state.
const VIEWS: [&str; 6] = [
    "summary",
    "nodes",
    "runtimes",
    "federation",
    "operations",
    "settings",
];

#[test]
#[ignore]
fn rasterize_primary_views() {
    for view in VIEWS {
        for dark in [false, true] {
            render_view(view, dark);
        }
    }
    println!("\nInspect /tmp/neonexus_truth_<view>_<theme>.png");
}

fn render_view(view: &str, dark: bool) {
    let temp = tempfile::tempdir().unwrap();
    let repository = fixture::seeded_workspace(&temp.path().join("neonexus.db"), dark, view);
    let mut app = NeoNexusApp::new(repository);

    let context = egui::Context::default();
    let raw = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..Default::default()
    };
    // Three passes: early frames grow the font atlas and settle layout, and the
    // last frame is the one rasterised. All texture deltas are kept because the
    // atlas is uploaded incrementally, one patch per frame.
    let frames: Vec<egui::FullOutput> = (0..3)
        .map(|_| context.run(raw.clone(), |ctx| app.render_headless_frame(ctx)))
        .collect();

    let theme = if dark { "dark" } else { "light" };
    let base = if dark {
        Color32::from_rgb(18, 17, 16)
    } else {
        Color32::from_rgb(244, 241, 236)
    };
    let image = raster::rasterize(&context, frames, SCREEN, base);
    let out = format!("/tmp/neonexus_truth_{view}_{theme}.png");
    image.save(std::path::Path::new(&out)).unwrap();
    println!("===== TRUTH [{view} / {theme}] -> {out} =====");
}
