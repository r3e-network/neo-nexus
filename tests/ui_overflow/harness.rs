//! Rendering a real frame and collecting what painted outside its clip.

use egui::{Pos2, Rect, Shape, Vec2};
use neo_nexus::{repository::Repository, NeoNexusApp};

/// The workbench's design window size (matches `run_native_app`).
const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);

/// Slack for hairline strokes and panel resize handles, which legitimately
/// paint a pixel or two outside their content region.
const TOLERANCE: f32 = 3.0;

/// A painted rectangle that extends past the clip region it belongs to.
pub(super) struct Overflow {
    pub(super) axis: &'static str,
    pub(super) amount: f32,
    pub(super) clip: Rect,
    pub(super) painted: Rect,
}

pub(super) fn overflows(view_key: &str, dark: bool) -> Vec<Overflow> {
    overflows_with(view_key, dark, true)
}

pub(super) fn overflows_with(view_key: &str, dark: bool, inspector: bool) -> Vec<Overflow> {
    overflows_in(view_key, dark, inspector, &[])
}

pub(super) fn overflows_in(
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

pub(super) fn report(view: &str, found: &[Overflow]) -> String {
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
