//! The walkthrough repeated at each UI density, and across a restart.
//!
//! Density changes every spacing constant at once, so a surface that fits at one
//! setting can overflow at another — these run the same primary-surface sweep
//! under both.

use super::*;

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
