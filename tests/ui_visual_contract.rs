//! Background-tier layering contract for the dark theme.
//!
//! A previous regression made the dark theme a "wall of near-black": the three
//! background tiers (window canvas < chrome panel < card) were all crammed into
//! the near-black brightness band, so 93.7% of the window read as black and the
//! whole workbench looked like a black block. This renders a real dark-mode
//! frame headlessly and asserts the three tiers are clearly separated by
//! luminance, so a future palette change cannot silently re-introduce the
//! black-wall regression. It is the objective, testable form of "the dark theme
//! is legible", complementing the geometry/typography/empty-state contracts.

use std::collections::HashMap;

use egui::{Color32, Pos2, Rect, Shape, Vec2};
use neo_nexus::{repository::Repository, NeoNexusApp};

const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);
/// Minimum luminance gap between adjacent background tiers. Tiers spaced less
/// than this read as the same surface (the black-wall failure mode).
const MIN_TIER_GAP: i32 = 10;

fn luminance(color: Color32) -> i32 {
    (color.r() as i32 + color.g() as i32 + color.b() as i32) / 3
}

/// Renders a dark-mode frame and returns every painted fill colour keyed by its
/// luminance, so the dominant background tiers can be compared.
fn dark_mode_fill_tiers() -> HashMap<(u8, u8, u8), i32> {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("neonexus.db");
    // Persist dark mode before constructing the app so it boots dark.
    let repo = Repository::open(&path).unwrap();
    repo.save_app_dark_mode(true).unwrap();
    drop(repo);
    let repository = Repository::open(&path).unwrap();
    let mut app = NeoNexusApp::new(repository);

    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
            ..Default::default()
        },
        |ctx| app.render_headless_frame(ctx),
    );

    // Sum the painted area of each distinct fill colour. Only sizable fills
    // count (ignore hairlines, text, and widget slivers).
    let mut area_by_color: HashMap<(u8, u8, u8), i32> = HashMap::new();
    for clipped in &output.shapes {
        let Shape::Rect(r) = &clipped.shape else {
            continue;
        };
        let rect_area = (r.rect.width() * r.rect.height()) as i32;
        if rect_area < 4000 || r.fill.a() == 0 {
            continue;
        }
        *area_by_color
            .entry((r.fill.r(), r.fill.g(), r.fill.b()))
            .or_default() += rect_area;
    }
    area_by_color
}

#[test]
fn dark_theme_background_tiers_are_clearly_separated() {
    let tiers = dark_mode_fill_tiers();
    // The three tiers must each be among the most-painted colours. Sort by area
    // and take the distinct luminances of the top fills.
    let mut by_area: Vec<((u8, u8, u8), i32)> = tiers.into_iter().collect();
    by_area.sort_by(|a, b| b.1.cmp(&a.1));

    let luminances: Vec<i32> = by_area
        .iter()
        .take(6)
        .map(|((r, g, b), _)| luminance(Color32::from_rgb(*r, *g, *b)))
        .collect();
    let max_lum = *luminances.iter().max().expect("painted fills");
    let min_lum = *luminances.iter().min().expect("painted fills");

    // The darkest tier (window canvas) and the lightest (card) must differ by a
    // clear margin — the black-wall regression collapsed this to near-zero.
    assert!(
        max_lum - min_lum >= MIN_TIER_GAP * 2,
        "dark-theme background tiers are too close: spread only {} (min spread {}); \
         the black-wall regression returns when tiers collapse into one band. \
         Top luminances: {:?}",
        max_lum - min_lum,
        MIN_TIER_GAP * 2,
        luminances,
    );
}
