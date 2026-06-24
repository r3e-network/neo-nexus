//! Headless empty-state verification of the native workbench.
//!
//! A fresh workspace renders with no nodes, no selection, and an empty fleet.
//! A professional, user-friendly application must turn each of those into an
//! actionable guidance message (not a blank panel), drawn at a legible size and
//! actually visible inside its panel. The same headless `egui::Context` path
//! used for geometry, contrast, and keyboard verification extracts every
//! rendered text run from the frame's `TextShape` galley jobs and asserts these
//! properties without a screenshot.

use std::collections::HashMap;

use egui::{Pos2, Rect, Shape, Vec2};
use neo_nexus::{repository::Repository, NeoNexusApp};

const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);
/// Smallest size a body/guidance message may render at and stay comfortably
/// legible. Below this, an empty-state message would be too quiet to read.
const MIN_LEGIBLE_SIZE: f32 = 11.0;

/// One rendered text run: its string, font size, its on-screen glyph bounds,
/// and the panel clip rect it is painted into.
struct RenderedText {
    text: String,
    size: f32,
    /// The glyph bounds translated to screen coordinates.
    bounds: Rect,
    /// The panel the text is clipped to; anything outside is not painted.
    clip: Rect,
}

/// Renders a fresh (no-data) workbench frame and collects every non-empty text
/// run with its size, screen bounds, and clip panel.
fn fresh_workbench_text() -> Vec<RenderedText> {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let mut app = NeoNexusApp::new(repository);

    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
            ..Default::default()
        },
        |ctx| app.render_headless_frame(ctx),
    );

    let mut out = Vec::new();
    for clipped in &output.shapes {
        let Shape::Text(ts) = &clipped.shape else {
            continue;
        };
        let text = ts.galley.job.text.clone();
        if text.trim().is_empty() {
            continue;
        }
        // The largest section font size is the run's effective size; egui mixes
        // sizes only rarely (icon glyph + label share the body size).
        let size = ts
            .galley
            .job
            .sections
            .iter()
            .map(|s| s.format.font_id.size)
            .fold(0.0_f32, f32::max);
        // The galley rect is relative to its text origin, and for right-aligned
        // text its min is negative (the glyphs sit left of the origin). Translate
        // it to screen coordinates by the run's origin.
        let bounds =
            Rect::from_min_size(ts.pos + ts.galley.rect.min.to_vec2(), ts.galley.rect.size());
        out.push(RenderedText {
            text,
            size,
            bounds,
            clip: clipped.clip_rect,
        });
    }
    out
}

#[test]
fn empty_states_render_actionable_guidance_not_blank_panels() {
    let texts = fresh_workbench_text();
    let by_text: HashMap<String, ()> = texts.iter().map(|t| (t.text.clone(), ())).collect();

    // Each empty state must offer an actionable next step, not just a label.
    for guidance in ["No nodes", "No node selected", "Empty fleet"] {
        assert!(
            by_text.contains_key(guidance),
            "missing empty-state heading {guidance:?}",
        );
    }
    for hint in [
        "Use New Node to define the first local runtime.",
        "Choose a node from Inventory.",
        "Use New Node to create the first local runtime.",
    ] {
        assert!(
            by_text.contains_key(hint),
            "missing empty-state guidance {hint:?}",
        );
    }
}

#[test]
fn every_rendered_text_is_legible_and_visible_in_its_panel() {
    for rt in fresh_workbench_text() {
        assert!(
            rt.size >= MIN_LEGIBLE_SIZE,
            "{:?} renders at {}pt (min {})",
            rt.text,
            rt.size,
            MIN_LEGIBLE_SIZE,
        );
        // The visible glyph bounds must overlap the panel the text is clipped to.
        // A run entirely outside its clip rect would not be painted at all.
        assert!(
            rt.bounds.intersects(rt.clip),
            "{:?} bounds {:?} do not overlap its panel {:?}",
            rt.text,
            rt.bounds,
            rt.clip,
        );
    }
}

#[test]
fn empty_state_heading_is_emphasised_above_body_caption_size() {
    // Empty-state headings (e.g. "Empty fleet") must read as more than a muted
    // caption: they should be at least as prominent as a section title.
    let texts = fresh_workbench_text();
    let heading = texts
        .iter()
        .find(|t| t.text == "Empty fleet")
        .expect("Empty fleet heading should render on the empty fleet view");
    assert!(
        heading.size >= 13.0,
        "empty-state heading {:?} is only {}pt; guidance should be emphasised",
        heading.text,
        heading.size,
    );
}
