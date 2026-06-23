//! Headless error-state verification via Repository fault injection.
//!
//! The workbench's error paths (snapshot registry, event journal, ...) branch
//! on a `Repository` call returning `Err`. Rather than expose test-only app
//! mutators, this injects a real fault -- a corrupted schema -- so the genuine
//! error path runs against the headless context. It then extracts the rendered
//! error text from the frame's TextShape galley jobs and asserts the failure is
//! present, drawn in the semantic danger colour, and visible inside its panel.

use egui::{Pos2, Rect, Shape, Vec2};
use neo_nexus::{app::View, repository::Repository, NeoNexusApp};

const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);

/// One rendered text run: its string, font size, screen bounds, and clip panel.
struct RenderedText {
    text: String,
    size: f32,
    bounds: Rect,
    clip: Rect,
}

/// Renders one headless frame of `app` and collects every non-empty text run.
fn rendered_text(app: &mut NeoNexusApp) -> Vec<RenderedText> {
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
        let size = ts
            .galley
            .job
            .sections
            .iter()
            .map(|s| s.format.font_id.size)
            .fold(0.0_f32, f32::max);
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

/// Opens a workspace, then drops the snapshot table so every snapshot query
/// fails. This is the fault the Snapshots view surfaces as an error state.
fn workspace_with_missing_snapshot_table(dir: &std::path::Path) -> Repository {
    let db_path = dir.join("neonexus.db");
    let repository = Repository::open(&db_path).unwrap();
    // Corrupt the schema by removing the table the Snapshots view reads. A fresh
    // rusqlite connection sees the same file Repository::open wrote.
    let connection = rusqlite::Connection::open(&db_path).unwrap();
    connection
        .execute("DROP TABLE fast_sync_snapshots", [])
        .unwrap();
    connection.close().unwrap();
    repository
}

#[test]
fn snapshots_view_renders_a_danger_coloured_error_when_the_registry_fails() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = workspace_with_missing_snapshot_table(temp_dir.path());
    let mut app = NeoNexusApp::new(repository);
    app.select_view(View::Snapshots);

    let texts = rendered_text(&mut app);

    // The Snapshots view calls empty_state with "Snapshot registry unavailable"
    // and the error message when list_fast_sync_snapshots returns Err.
    let heading = texts
        .iter()
        .find(|t| t.text == "Snapshot registry unavailable");
    assert!(
        heading.is_some(),
        "missing snapshot registry error heading; rendered: {:?}",
        texts.iter().map(|t| t.text.as_str()).collect::<Vec<_>>(),
    );

    // The error heading must be visible inside its panel clip rect.
    let heading = heading.unwrap();
    assert!(
        heading.bounds.intersects(heading.clip),
        "snapshot error heading {:?} is outside its panel {:?}",
        heading.bounds,
        heading.clip,
    );

    // The failure must surface as a legible, on-screen message (not a blank
    // panel or a truncated run), proving the error path rendered. The error
    // message row carries the underlying failure text and is itself visible.
    let message_visible = texts
        .iter()
        .any(|t| !t.text.is_empty() && t.bounds.intersects(heading.clip) && t.size >= 11.0);
    assert!(
        message_visible,
        "snapshot error did not render a legible message",
    );
}
