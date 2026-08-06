//! Every row of a paginated list must be reachable with the pagination bar.
//!
//! Page sizes are derived from the panel height at the render site, so a list
//! usually has more pages than the fixed page-size constant implies. The
//! flow-level reconcilers that run each frame out of `eframe::App::update` must
//! not clamp the page index against that constant, or the operator is snapped
//! back and the rows past it can never be shown.
//!
//! This drives the real `update()` rather than `render_headless_frame`, because
//! the per-frame reconciliation only runs on the former.

#[path = "ui_visual_truth/fixture.rs"]
mod fixture;

use eframe::App as _;
use egui::{Event, PointerButton, Pos2, Rect, Vec2};
use neo_nexus::NeoNexusApp;

const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);
/// The inventory side panel's clip starts at the navigation sidebar's edge.
const INVENTORY_CLIP_X: f32 = 214.0;

fn base() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..Default::default()
    }
}

/// One frame's worth of inventory-column facts: the row ports painted (one per
/// node row), the "N items" total, the page counter, and the Next button.
#[derive(Default)]
struct Scan {
    ports: Vec<String>,
    items: Option<usize>,
    counter: Option<String>,
    next: Option<Pos2>,
}

fn scan(out: &egui::FullOutput) -> Scan {
    let mut found = Scan::default();
    for c in &out.shapes {
        let egui::Shape::Text(t) = &c.shape else {
            continue;
        };
        if (c.clip_rect.min.x - INVENTORY_CLIP_X).abs() > 1.0 {
            continue;
        }
        let s = t.galley.text();
        if s == "Next" {
            found.next = Some(t.visual_bounding_rect().center());
        } else if let Some(total) = s.strip_suffix(" items") {
            found.items = total.trim().parse().ok();
        } else if let Some(port) = s.strip_prefix(':') {
            if port.chars().all(|c| c.is_ascii_digit()) {
                found.ports.push(s.to_string());
            }
        } else if s.contains('/') && s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            found.counter = Some(s.to_string());
        }
    }
    found
}

fn click_at(pos: Pos2) -> egui::RawInput {
    let mut input = base();
    let button = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    input.events = vec![Event::PointerMoved(pos), button(true), button(false)];
    input
}

#[test]
fn every_inventory_node_is_reachable_by_paging_forward() {
    let temp = tempfile::tempdir().unwrap();
    let repo = fixture::seeded_workspace(&temp.path().join("n.db"), false, "nodes");
    let mut app = NeoNexusApp::new(repo);
    let context = egui::Context::default();
    let mut frame = eframe::Frame::_new_kittest();

    let mut reached: Vec<String> = Vec::new();
    let mut expected = 0usize;
    let mut counters: Vec<String> = Vec::new();

    // Bounded walk: click Next until the page counter stops advancing.
    for _ in 0..12 {
        let out = context.run(base(), |c| app.update(c, &mut frame));
        let found = scan(&out);
        expected = found.items.unwrap_or(expected);
        reached.extend(found.ports);
        let Some(counter) = found.counter else { break };
        if counters.last() == Some(&counter) {
            break;
        }
        counters.push(counter);
        let Some(next) = found.next else { break };
        let _ = context.run(click_at(next), |c| app.update(c, &mut frame));
    }

    reached.sort();
    reached.dedup();
    assert!(expected > 0, "inventory reported no items");
    assert_eq!(
        reached.len(),
        expected,
        "only {} of {expected} inventory nodes could be reached by paging forward \
         (pages visited: {counters:?}, ports seen: {reached:?})",
        reached.len(),
    );
}
