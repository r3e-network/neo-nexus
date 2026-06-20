use super::super::super::*;
use super::neo_rs_backup_node;

#[test]
fn backup_validation_cli_reports_valid_backup_summary() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    repository.create_node(neo_rs_backup_node("backupable neo-rs", 10332, 10333, 10334))?;

    let export =
        WorkspaceBackupExporter::write(&repository, temp_dir.path().join("backups"), "test")?;
    let backup_arg = export.path.display().to_string();
    let action = action_from_args(["neo-nexus", "--validate-backup", &backup_arg])?;

    assert!(
        matches!(action, CliAction::Print(text) if text.contains("backup-validation: ok") && text.contains("nodes: 1") && text.contains("schema-version: 7"))
    );
    Ok(())
}

#[test]
fn backup_validation_json_cli_reports_machine_readable_summary() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    repository.create_node(neo_rs_backup_node("backupable neo-rs", 10332, 10333, 10334))?;

    let export =
        WorkspaceBackupExporter::write(&repository, temp_dir.path().join("backups"), "test")?;
    let backup_arg = export.path.display().to_string();
    let action = action_from_args(["neo-nexus", "--validate-backup-json", &backup_arg])?;

    let CliAction::Print(text) = action else {
        anyhow::bail!("expected JSON backup validation action");
    };
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["validation"]["schema_version"], 7);
    assert_eq!(value["validation"]["application_version"], "test");
    assert_eq!(value["validation"]["node_count"], 1);
    assert_eq!(value["validation"]["source_path"], backup_arg);
    Ok(())
}

#[test]
fn backup_validation_cli_rejects_unsafe_workspace_settings() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let backup_path = temp_dir.path().join("unsafe-backup.json");
    std::fs::write(
        &backup_path,
        r#"{
          "schema_version": 5,
          "application": "NeoNexus",
          "application_version": "test",
          "exported_at_unix": 1800000000,
          "workspace_settings": [
            { "key": "unsupported.secret", "value": "do-not-import" }
          ],
          "runtime_catalog_profiles": [],
          "runtime_signer_profiles": [],
          "fast_sync_snapshots": [],
          "nodes": [],
          "events": []
        }"#,
    )?;

    let backup_arg = backup_path.display().to_string();
    let error = action_from_args(["neo-nexus", "--validate-backup", &backup_arg])
        .expect_err("unsafe backup should be rejected");
    assert!(error.to_string().contains("unsupported workspace setting"));
    Ok(())
}
