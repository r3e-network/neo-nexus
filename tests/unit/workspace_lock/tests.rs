use super::WorkspaceSupervisorLock;

#[test]
fn one_workspace_has_exactly_one_long_running_supervisor() {
    let workspace = tempfile::tempdir().unwrap();
    let first = WorkspaceSupervisorLock::acquire(workspace.path()).unwrap();
    let duplicate = WorkspaceSupervisorLock::acquire(workspace.path());
    assert!(duplicate.is_err());
    assert!(duplicate
        .unwrap_err()
        .to_string()
        .contains("already has an active NeoNexus supervision server"));

    drop(first);
    WorkspaceSupervisorLock::acquire(workspace.path())
        .expect("the operating-system lock is released when the server stops");
}
