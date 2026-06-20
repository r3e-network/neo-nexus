use super::super::*;

#[test]
fn backup_export_cli_rejects_missing_workspace_database() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_arg = temp_dir.path().join("missing.db").display().to_string();
    let output_arg = temp_dir.path().join("backups").display().to_string();

    let error = action_from_args(["neo-nexus", "--export-backup", &db_arg, &output_arg])
        .expect_err("missing database should not be silently created");

    assert!(error.to_string().contains("does not exist"));
    Ok(())
}

#[test]
fn event_journal_export_cli_rejects_missing_workspace_database() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_arg = temp_dir.path().join("missing.db").display().to_string();
    let report_arg = temp_dir.path().join("events").display().to_string();

    let error = action_from_args(["neo-nexus", "--export-event-journal", &db_arg, &report_arg])
        .expect_err("missing event journal database should not be silently created");

    assert!(error.to_string().contains("does not exist"));
    Ok(())
}

#[test]
fn node_config_export_cli_rejects_missing_workspace_database() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_arg = temp_dir.path().join("missing.db").display().to_string();
    let output_arg = temp_dir.path().join("configs").display().to_string();

    let error = action_from_args(["neo-nexus", "--export-node-configs", &db_arg, &output_arg])
        .expect_err("missing config export database should not be silently created");

    assert!(error.to_string().contains("does not exist"));
    Ok(())
}

#[test]
fn support_bundle_cli_rejects_missing_workspace_database() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_arg = temp_dir.path().join("missing.db").display().to_string();
    let output_arg = temp_dir.path().join("support").display().to_string();

    let error = action_from_args(["neo-nexus", "--export-support-bundle", &db_arg, &output_arg])
        .expect_err("missing support bundle database should not be silently created");

    assert!(error.to_string().contains("does not exist"));
    Ok(())
}

#[test]
fn workspace_integrity_cli_rejects_missing_workspace_database() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_arg = temp_dir.path().join("missing.db").display().to_string();

    let error = action_from_args(["neo-nexus", "--workspace-integrity", &db_arg])
        .expect_err("missing integrity database should not be silently created");

    assert!(error.to_string().contains("does not exist"));
    Ok(())
}
