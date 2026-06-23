use super::*;

/// A fresh workspace stores no section preference: `load_workspace_section`
/// returns `None` for any key, which is the contract the app relies on to fall
/// back to each page's default sub-tab on a clean launch.
#[test]
fn fresh_workspace_has_no_persisted_sub_tab() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    assert_eq!(
        repository
            .load_workspace_section("workspace.section.operations")
            .unwrap(),
        None,
    );
    assert_eq!(
        repository
            .load_workspace_section("workspace.section.settings")
            .unwrap(),
        None,
    );
}

/// Each sub-tab preference survives a save + reload round-trip through a real
/// SQLite workspace, and an overwrite replaces the previous value. The setting
/// key is an opaque string from the repository's perspective, so the same
/// storage path serves every dense page.
#[test]
fn sub_tab_preferences_round_trip_through_the_repository() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let key = "workspace.section.operations";

    repository.save_workspace_section(key, "readiness").unwrap();
    assert_eq!(
        repository.load_workspace_section(key).unwrap().as_deref(),
        Some("readiness"),
    );

    // Overwriting advances to the operator's latest selection.
    repository.save_workspace_section(key, "journal").unwrap();
    assert_eq!(
        repository.load_workspace_section(key).unwrap().as_deref(),
        Some("journal"),
    );

    // Distinct dense pages do not collide: each key is an independent slot.
    repository
        .save_workspace_section("workspace.section.settings", "release")
        .unwrap();
    assert_eq!(
        repository
            .load_workspace_section("workspace.section.settings")
            .unwrap()
            .as_deref(),
        Some("release"),
    );
    assert_eq!(
        repository.load_workspace_section(key).unwrap().as_deref(),
        Some("journal"),
    );
}
