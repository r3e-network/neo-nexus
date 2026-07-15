//! Headless operator walkthrough for v3.1 density + primary surfaces.
//!
//! Replaces a manual GUI click-through:
//! 1. Seed a multi-node workspace.
//! 2. Paint Home (inventory visible) under Comfortable and Compact.
//! 3. Assert Compact densifies controls; chrome 60/28/212 stays fixed.
//! 4. Visit every primary nav destination under Compact without panicking.
//! 5. Reload Compact from SQLite (Settings density persist path).

use std::{collections::HashSet, path::PathBuf};

use egui::{Pos2, Rect, Vec2};
use neo_nexus::{
    app::View,
    repository::Repository,
    types::{Network, NewNode, NodeType},
    NeoNexusApp,
};

const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);
const CHROME_HEADER: f32 = 60.0;
const CHROME_STATUS: f32 = 28.0;
const CHROME_SIDEBAR: f32 = 212.0;

/// Six primaries operators hit from the sidebar.
const PRIMARIES: &[&str] = &[
    "summary",
    "nodes",
    "runtimes",
    "federation",
    "operations",
    "settings",
];

struct PaintSnapshot {
    rects: Vec<Rect>,
    interact_y: f32,
    button_pad_y: f32,
    item_spacing_y: f32,
    shape_count: usize,
}

fn seed_nodes(repository: &Repository) {
    for (i, (name, node_type)) in [
        ("validator-a", NodeType::NeoGo),
        ("rpc-b", NodeType::NeoCli),
        ("relay-c", NodeType::NeoRs),
    ]
    .into_iter()
    .enumerate()
    {
        let base = 10332 + (i as u16) * 10;
        repository
            .create_node(NewNode {
                name: name.to_string(),
                node_type,
                network: Network::Testnet,
                binary_path: PathBuf::from("/usr/local/bin/node"),
                args: Vec::new(),
                runtime_version: "latest".to_string(),
                storage_engine: node_type.default_storage_engine(),
                rpc_port: base,
                p2p_port: base + 1,
                ws_port: Some(base + 2),
            })
            .expect("seed node");
    }
}

fn open_app(density: &str, view: &str) -> (tempfile::TempDir, NeoNexusApp) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    seed_nodes(&repository);
    repository
        .save_app_ui_density(density)
        .expect("save density");
    repository
        .save_workspace_last_view(view)
        .expect("save view");
    let app = NeoNexusApp::new(repository);
    (temp_dir, app)
}

fn paint(app: &mut NeoNexusApp) -> PaintSnapshot {
    let ctx = egui::Context::default();
    let raw = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..Default::default()
    };

    let mut interact_y = 0.0;
    let mut button_pad_y = 0.0;
    let mut item_spacing_y = 0.0;
    let output = ctx.run(raw, |ctx| {
        app.render_headless_frame(ctx);
        let spacing = &ctx.style().spacing;
        interact_y = spacing.interact_size.y;
        button_pad_y = spacing.button_padding.y;
        item_spacing_y = spacing.item_spacing.y;
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

    PaintSnapshot {
        rects,
        interact_y,
        button_pad_y,
        item_spacing_y,
        shape_count: output.shapes.len(),
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
        .expect("panel")
}

fn assert_chrome(snap: &PaintSnapshot, label: &str) {
    let header = panel_near(&snap.rects, SCREEN.x, CHROME_HEADER);
    let status = panel_near(&snap.rects, SCREEN.x, CHROME_STATUS);
    let sidebar = panel_near(&snap.rects, CHROME_SIDEBAR, SCREEN.y);
    assert!(
        (header.height() - CHROME_HEADER).abs() < 1.0,
        "{label}: header {header:?}"
    );
    assert!(
        (status.height() - CHROME_STATUS).abs() < 1.0,
        "{label}: status {status:?}"
    );
    assert!(
        (sidebar.width() - CHROME_SIDEBAR).abs() < 1.0,
        "{label}: sidebar {sidebar:?}"
    );
    assert!(
        snap.shape_count > 20,
        "{label}: expected a painted workbench, got {} shapes",
        snap.shape_count
    );
}

#[test]
fn density_toggle_keeps_chrome_and_densifies_controls() {
    let (_tmp_c, mut comfortable_app) = open_app("comfortable", "summary");
    let comfortable = paint(&mut comfortable_app);
    assert_chrome(&comfortable, "comfortable/home");
    assert!(
        (comfortable.interact_y - 28.0).abs() < 0.5,
        "comfortable interact_y {}",
        comfortable.interact_y
    );

    let (_tmp_k, mut compact_app) = open_app("compact", "summary");
    let compact = paint(&mut compact_app);
    assert_chrome(&compact, "compact/home");
    assert!(
        (compact.interact_y - 24.0).abs() < 0.5,
        "compact interact_y {}",
        compact.interact_y
    );
    assert!(
        compact.interact_y < comfortable.interact_y,
        "Compact must densify controls"
    );
    assert!(
        compact.button_pad_y < comfortable.button_pad_y,
        "Compact button pad denser"
    );
    assert!(
        compact.item_spacing_y < comfortable.item_spacing_y,
        "Compact item spacing denser"
    );

    // Inventory panel present on Home for both densities (seeded fleet).
    let inv_c = panel_near(&comfortable.rects, 248.0, SCREEN.y);
    let inv_k = panel_near(&compact.rects, 248.0, SCREEN.y);
    assert!(
        inv_c.width() > 180.0 && inv_k.width() > 180.0,
        "inventory panel should paint with seeded nodes"
    );
}

#[test]
fn compact_walks_all_primary_surfaces_without_panic() {
    for view_key in PRIMARIES {
        let (_tmp, mut app) = open_app("compact", view_key);
        let snap = paint(&mut app);
        assert_chrome(&snap, &format!("compact/{view_key}"));
        assert!(
            View::from_persist_key(view_key).is_some(),
            "unknown view key {view_key}"
        );
    }
}

#[test]
fn compact_preference_survives_restart() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db = temp_dir.path().join("neonexus.db");
    {
        let repository = Repository::open(&db).unwrap();
        seed_nodes(&repository);
        repository.save_app_ui_density("compact").unwrap();
        repository.save_workspace_last_view("settings").unwrap();
        let mut app = NeoNexusApp::new(repository);
        let snap = paint(&mut app);
        assert_chrome(&snap, "settings-compact-first");
        assert!((snap.interact_y - 24.0).abs() < 0.5);
    }
    // Re-open same DB — density must reload as Compact (Storage preference path).
    let repository = Repository::open(&db).unwrap();
    let mut app = NeoNexusApp::new(repository);
    let snap = paint(&mut app);
    assert_chrome(&snap, "settings-compact-reload");
    assert!(
        (snap.interact_y - 24.0).abs() < 0.5,
        "reloaded density should stay compact, got interact_y {}",
        snap.interact_y
    );
}

#[test]
fn nodes_and_operations_paint_with_fleet_under_both_densities() {
    for density in ["comfortable", "compact"] {
        for view in ["nodes", "operations"] {
            let (_tmp, mut app) = open_app(density, view);
            let snap = paint(&mut app);
            assert_chrome(&snap, &format!("{density}/{view}"));
            let inv = panel_near(&snap.rects, 248.0, SCREEN.y);
            assert!(
                inv.width() > 180.0,
                "{density}/{view}: inventory should show"
            );
        }
    }
}
