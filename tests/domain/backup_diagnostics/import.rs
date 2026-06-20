use super::*;

#[path = "import/restore_assertions.rs"]
mod restore_assertions;
#[path = "import/source_workspace.rs"]
mod source_workspace;
#[path = "import/validation_assertions.rs"]
mod validation_assertions;

#[test]
fn workspace_backup_imports_nodes_safely_and_updates_existing_records() {
    let temp_dir = tempfile::tempdir().unwrap();
    let (backup, node_id) = source_workspace::backup_with_full_workspace(temp_dir.path());
    let target = Repository::open(temp_dir.path().join("target.db")).unwrap();

    let imported = WorkspaceBackupImporter::import(&target, &backup).unwrap();
    restore_assertions::assert_first_import(&target, &imported, &node_id);

    let imported_again = WorkspaceBackupImporter::import(&target, &backup).unwrap();
    restore_assertions::assert_second_import(&target, &imported_again);

    validation_assertions::assert_rejects_unsafe_backup_shapes(&target, &backup);
}
