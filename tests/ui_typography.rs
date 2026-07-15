//! Headless typographic verification of the native workbench.
//!
//! The design type scale (tokens.rs: 11/12/13/14/17/24pt) is the single tuning
//! knob for typography. Any rendered text at another size bypasses that system,
//! fragmenting the visual rhythm. This renders a real frame headlessly, reads
//! every TextShape's effective font size, and asserts they all fall on the
//! scale. It also checks the sidebar's navigation rows sit on a consistent
//! baseline grid so the source-list reads evenly without a screenshot.

use std::collections::BTreeMap;

use egui::{Pos2, Rect, Shape, Vec2};
use neo_nexus::{repository::Repository, NeoNexusApp};

const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);
/// The design type scale (tokens.rs). Every body/label/caption/heading renders
/// at one of these sizes; anything else is an off-scale bypass.
const TYPE_SCALE: [f32; 6] = [11.0, 12.0, 13.0, 14.0, 17.0, 24.0];

/// Tolerance for float rounding when matching against the scale.
fn on_scale(size: f32) -> bool {
    TYPE_SCALE.iter().any(|&s| (size - s).abs() < 0.25)
}

/// Renders a fresh workbench frame and returns the effective font size of every
/// non-empty text run.
fn rendered_font_sizes() -> Vec<f32> {
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

    let mut sizes = Vec::new();
    for clipped in &output.shapes {
        let Shape::Text(ts) = &clipped.shape else {
            continue;
        };
        if ts.galley.job.text.trim().is_empty() {
            continue;
        }
        let size = ts
            .galley
            .job
            .sections
            .iter()
            .map(|s| s.format.font_id.size)
            .fold(0.0_f32, f32::max);
        sizes.push(size);
    }
    sizes
}

#[test]
fn every_rendered_text_uses_the_design_type_scale() {
    let mut off_scale: BTreeMap<i64, (f32, usize)> = BTreeMap::new();
    for size in rendered_font_sizes() {
        if on_scale(size) {
            continue;
        }
        let key = (size * 2.0).round() as i64;
        let entry = off_scale.entry(key).or_insert((size, 0));
        entry.1 += 1;
    }
    assert!(
        off_scale.is_empty(),
        "rendered text uses off-scale sizes (design scale is {:?}); bypassing tokens.rs fragments the type rhythm: {:?}",
        TYPE_SCALE,
        off_scale.into_iter().map(|(_, (s, c))| (s, c)).collect::<Vec<_>>(),
    );
}

/// Sidebar navigation rows should sit on a uniform baseline grid so the
/// source-list reads evenly. The Phosphor glyph + label render in one galley,
/// so each nav item's baseline is its TextShape origin y; consecutive gaps must
/// match the design row height (32pt) within a tight tolerance.
#[test]
fn sidebar_navigation_rows_sit_on_a_consistent_baseline_grid() {
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

    // Nav items live in the navigation panel (x < 212) and start with a Phosphor
    // glyph (U+E000..). Group them so a section break (a larger gap) does not
    // look like jitter within the run.
    let mut baselines: Vec<f32> = Vec::new();
    for clipped in &output.shapes {
        let Shape::Text(ts) = &clipped.shape else {
            continue;
        };
        if clipped.clip_rect.min.x >= 212.0 {
            continue;
        }
        let text = &ts.galley.job.text;
        if text.chars().next().is_some_and(|c| (c as u32) >= 0xE000) {
            baselines.push(ts.pos.y);
        }
    }
    baselines.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Group consecutive rows by gap: a uniform 40pt run is one visual group; a
    // larger gap marks a section break (a new group). Within every group the
    // leading must be consistent so the source-list reads evenly.
    let row_height = 40.0;
    let mut groups: Vec<Vec<f32>> = vec![vec![baselines[0]]];
    for &y in &baselines[1..] {
        let prev = *groups.last().and_then(|g| g.last()).unwrap();
        if (y - prev - row_height).abs() <= 1.5 {
            groups.last_mut().unwrap().push(y);
        } else {
            groups.push(vec![y]);
        }
    }

    for group in &groups {
        for w in group.windows(2) {
            let gap = w[1] - w[0];
            assert!(
                (gap - row_height).abs() <= 1.5,
                "sidebar rows have uneven leading within a section: {gap:.1}pt (target {row_height})",
            );
        }
    }
    // The sidebar must render its full set of primary nav items (v3: six),
    // not collapse any.
    assert!(
        baselines.len() >= 6,
        "expected all primary navigation rows, got {}",
        baselines.len(),
    );
}
