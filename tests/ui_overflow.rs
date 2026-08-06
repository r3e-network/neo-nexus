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
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&path).unwrap();
    repository.save_app_dark_mode(dark).unwrap();
    repository.save_app_inspector_visible(true).unwrap();
    repository.save_workspace_last_view(view_key).unwrap();
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
    found
        .iter()
        .map(|overflow| {
            format!(
                "\n  {view}: {:.1}pt past the {} edge — painted {:.0}..{:.0}, clip {:.0}..{:.0}",
                overflow.amount,
                overflow.axis,
                overflow.painted.min.x,
                overflow.painted.max.x,
                overflow.clip.min.x,
                overflow.clip.max.x,
            )
        })
        .collect()
}

#[test]
fn no_surface_paints_outside_its_own_column() {
    let mut failures = String::new();
    for view in ["summary", "nodes", "runtimes", "federation", "operations"] {
        let found = overflows(view, false);
        failures.push_str(&report(view, &found));
    }
    assert!(
        failures.is_empty(),
        "surfaces paint outside the column that will clip them, so their \
         content is silently truncated:{failures}",
    );
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
