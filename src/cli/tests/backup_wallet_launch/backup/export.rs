use super::super::super::*;
use super::neo_rs_backup_node;

#[test]
fn backup_export_cli_writes_workspace_backup_summary() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let output_dir = temp_dir.path().join("backups");
    let repository = Repository::open(&db_path)?;
    repository.create_node(neo_rs_backup_node("exportable neo-rs", 10332, 10333, 10334))?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let output_arg = output_dir.display().to_string();
    let action = action_from_args(["neo-nexus", "--export-backup", &db_arg, &output_arg])?;

    assert!(
        matches!(action, CliAction::Print(text) if text.contains("backup-export: ok") && text.contains("schema-version: 7") && text.contains("nodes: 1") && text.contains("bytes-written:"))
    );
    let backup_files = std::fs::read_dir(&output_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    assert_eq!(backup_files.len(), 1);
    let validation = WorkspaceBackupImporter::validate_path(&backup_files[0])?;
    assert_eq!(validation.node_count, 1);
    assert_eq!(validation.schema_version, 7);
    Ok(())
}

#[test]
fn backup_export_json_cli_writes_backup_and_reports_machine_readable_summary() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let output_dir = temp_dir.path().join("backups-json");
    let repository = Repository::open(&db_path)?;
    repository.create_node(neo_rs_backup_node("exportable neo-rs", 10332, 10333, 10334))?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let output_arg = output_dir.display().to_string();
    let action = action_from_args(["neo-nexus", "--export-backup-json", &db_arg, &output_arg])?;

    let CliAction::Print(text) = action else {
        anyhow::bail!("expected JSON backup export action");
    };
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["export"]["schema_version"], 7);
    assert_eq!(
        value["export"]["application_version"],
        env!("CARGO_PKG_VERSION")
    );
    assert_eq!(value["export"]["node_count"], 1);
    assert!(value["export"]["bytes_written"]
        .as_u64()
        .is_some_and(|bytes| bytes > 0));

    let backup_path = value["export"]["path"]
        .as_str()
        .context("missing exported backup path")?;
    assert!(std::path::Path::new(backup_path).is_file());
    let validation = WorkspaceBackupImporter::validate_path(backup_path)?;
    assert_eq!(validation.node_count, 1);
    Ok(())
}
