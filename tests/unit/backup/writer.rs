use super::*;
use crate::{
    backup::{WorkspaceBackupExporter, WorkspaceBackupImporter},
    repository::Repository,
};

#[test]
fn same_second_exports_never_overwrite_previous_backup() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("test.db")).unwrap();
    let output = dir.path().join("backups");
    let mut backup = WorkspaceBackupExporter::snapshot(&repository, "first", 123).unwrap();
    let first = write_backup_file(&backup, &output, 123).unwrap();
    let original = fs::read(&first.path).unwrap();
    backup.application_version = "second".into();
    let second = write_backup_file(&backup, &output, 123).unwrap();
    assert_ne!(first.path, second.path);
    assert_eq!(fs::read(&first.path).unwrap(), original);
    assert_eq!(
        WorkspaceBackupImporter::read(second.path)
            .unwrap()
            .application_version,
        "second"
    );
}
