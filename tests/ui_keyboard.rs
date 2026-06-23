//! Headless keyboard-reachability verification of the native workbench.
//!
//! The same headless `egui::Context` path used for geometry is used here to
//! prove the workbench is operable without a mouse: each frame injects a Tab
//! key event, and `Memory::focused()` records which interactive widget holds
//! focus. A professional, user-friendly application must reach its primary
//! controls by keyboard, and this asserts it concretely rather than by eye.

use std::collections::HashSet;

use egui::{Event, Id, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
use neo_nexus::{repository::Repository, NeoNexusApp};

/// The workbench's design window size (matches `run_native_app`).
const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);

/// Runs `frames` headless renders, injecting one Tab keypress per frame from
/// the second frame onward, and returns the sequence of focused widget ids.
fn tab_focus_path(frames: usize) -> Vec<Option<Id>> {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let mut app = NeoNexusApp::new(repository);

    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, SCREEN);

    // First frame seeds the widget tree with no input.
    let _ = ctx.run(
        RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ctx| app.render_headless_frame(ctx),
    );

    let mut focused = Vec::new();
    for _ in 0..frames {
        let _ = ctx.run(
            RawInput {
                screen_rect: Some(screen),
                events: vec![Event::Key {
                    key: Key::Tab,
                    pressed: true,
                    repeat: false,
                    physical_key: None,
                    modifiers: Modifiers::default(),
                }],
                ..Default::default()
            },
            |ctx| app.render_headless_frame(ctx),
        );
        focused.push(ctx.memory(|m| m.focused()));
    }
    focused
}

#[test]
fn tab_reaches_many_distinct_interactive_widgets() {
    // A professional workbench exposes its primary controls by keyboard. The
    // Summary view alone has the sidebar, inventory, inspector, and workspace
    // controls, so Tab must walk well past a handful of widgets.
    let focused = tab_focus_path(40);
    let distinct: HashSet<Option<Id>> = focused.iter().copied().collect();
    assert!(
        distinct.len() >= 10,
        "Tab only reached {} distinct widgets; the workbench should be fully keyboard-navigable",
        distinct.len(),
    );
}

#[test]
fn tab_does_not_get_stuck_on_a_single_widget() {
    // If Tab were a no-op (e.g. no focusable widgets), every sample would be
    // identical. Advancing past the first widget proves the focus chain moves.
    let focused = tab_focus_path(20);
    let first = focused.first().copied().flatten();
    let advanced = focused.iter().any(|id| *id != first);
    assert!(
        advanced,
        "Tab never moved focus off the first widget; the focus chain may be frozen",
    );
}

#[test]
fn tab_visits_widgets_rather_than_losing_focus_entirely() {
    // Across a tab cycle, focus should land on real widgets most of the time,
    // not collapse to None. A majority-None result would signal broken focus.
    let focused = tab_focus_path(40);
    let on_widget = focused.iter().filter(|id| id.is_some()).count();
    assert!(
        on_widget >= 30,
        "focus was on a widget only {on_widget} of 40 Tab presses; keyboard focus is unstable",
    );
}
