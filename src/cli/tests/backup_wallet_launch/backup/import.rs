use super::super::super::*;
use super::neo_rs_backup_node;

#[test]
fn backup_import_cli_restores_workspace_backup_summary() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let source = Repository::open(temp_dir.path().join("source.db"))?;
    source.create_node(neo_rs_backup_node("importable neo-rs", 10332, 10333, 10334))?;
    let export = WorkspaceBackupExporter::write(&source, temp_dir.path().join("backups"), "test")?;
    drop(source);

    let target_path = temp_dir.path().join("target.db");
    let target_arg = target_path.display().to_string();
    let backup_arg = export.path.display().to_string();
    let action = action_from_args(["neo-nexus", "--import-backup", &target_arg, &backup_arg])?;

    assert!(
        matches!(action, CliAction::Print(text) if text.contains("backup-import: ok") && text.contains("target-database:") && text.contains("created-nodes: 1") && text.contains("updated-nodes: 0") && text.contains("schema-version: 7"))
    );
    let target = Repository::open(target_path)?;
    let nodes = target.list_nodes()?;
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].name, "importable neo-rs");
    assert_eq!(nodes[0].status, NodeStatus::Stopped);
    Ok(())
}

#[test]
fn backup_import_json_cli_reports_machine_readable_summary() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let source = Repository::open(temp_dir.path().join("source.db"))?;
    source.create_node(neo_rs_backup_node(
        "json importable neo-rs",
        11332,
        11333,
        11334,
    ))?;
    let export = WorkspaceBackupExporter::write(&source, temp_dir.path().join("backups"), "test")?;
    drop(source);

    let target_path = temp_dir.path().join("target-json.db");
    let target_arg = target_path.display().to_string();
    let backup_arg = export.path.display().to_string();
    let action = action_from_args([
        "neo-nexus",
        "--import-backup-json",
        &target_arg,
        &backup_arg,
    ])?;

    let CliAction::Print(text) = action else {
        anyhow::bail!("expected JSON backup import action");
    };
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["target_database"], target_arg);
    assert_eq!(value["import"]["source_path"], backup_arg);
    assert_eq!(value["import"]["schema_version"], 7);
    assert_eq!(value["import"]["created_nodes"], 1);
    assert_eq!(value["import"]["updated_nodes"], 0);
    assert_eq!(value["import"]["plugin_state_count"], 0);
    assert_eq!(Repository::open(target_path)?.list_nodes()?.len(), 1);
    Ok(())
}

#[test]
fn backup_import_cli_rejects_active_target_nodes() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let source = Repository::open(temp_dir.path().join("source.db"))?;
    source.create_node(neo_rs_backup_node("source neo-rs", 10332, 10333, 10334))?;
    let export = WorkspaceBackupExporter::write(&source, temp_dir.path().join("backups"), "test")?;
    drop(source);

    let target_path = temp_dir.path().join("active-target.db");
    let target = Repository::open(&target_path)?;
    let active = target.create_node(neo_rs_backup_node("active target", 20332, 20333, 20334))?;
    target.update_node_status(&active.id, NodeStatus::Running, Some(4242))?;
    drop(target);

    let target_arg = target_path.display().to_string();
    let backup_arg = export.path.display().to_string();
    let error = action_from_args(["neo-nexus", "--import-backup", &target_arg, &backup_arg])
        .expect_err("active target nodes should block backup import");

    assert!(error.to_string().contains("active target"));
    assert!(error.to_string().contains("stop active nodes"));
    Ok(())
}
