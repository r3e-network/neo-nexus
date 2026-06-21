use super::*;

pub(in crate::cli::actions) fn validate_backup_text(args: &[String]) -> Result<String> {
    require_arg_count(args, 3, "--validate-backup")?;
    let validation = WorkspaceBackupImporter::validate_path(PathBuf::from(&args[2]))?;
    Ok(validation.to_cli_text())
}

pub(in crate::cli::actions) fn validate_backup_json_text(args: &[String]) -> Result<String> {
    require_arg_count(args, 3, "--validate-backup-json")?;
    let validation = WorkspaceBackupImporter::validate_path(PathBuf::from(&args[2]))?;
    backup_validation_json_text(&validation)
}

pub(in crate::cli::actions) fn export_backup_text(args: &[String]) -> Result<String> {
    let export = export_backup(args, "--export-backup")?;
    Ok(export.to_cli_text())
}

pub(in crate::cli::actions) fn export_backup_json_text(args: &[String]) -> Result<String> {
    let export = export_backup(args, "--export-backup-json")?;
    backup_export_json_text(&export)
}

pub(in crate::cli::actions) fn import_backup_text(args: &[String]) -> Result<String> {
    let (db_path, import) = import_backup(args, "--import-backup")?;
    Ok(import.to_cli_text_with_target(Some(&db_path)))
}

pub(in crate::cli::actions) fn import_backup_json_text(args: &[String]) -> Result<String> {
    let (db_path, import) = import_backup(args, "--import-backup-json")?;
    backup_import_json_text(&db_path, &import)
}

fn export_backup(args: &[String], option: &str) -> Result<WorkspaceBackupExport> {
    require_arg_count(args, 4, option)?;
    let db_path = PathBuf::from(&args[2]);
    if !db_path.is_file() {
        anyhow::bail!(
            "workspace database {} does not exist; pass an existing neonexus.db",
            db_path.display()
        );
    }
    let repository = Repository::open(&db_path)
        .with_context(|| format!("failed to open workspace database {}", db_path.display()))?;
    WorkspaceBackupExporter::write(
        &repository,
        PathBuf::from(&args[3]),
        env!("CARGO_PKG_VERSION"),
    )
}

fn import_backup(args: &[String], option: &str) -> Result<(PathBuf, WorkspaceBackupImport)> {
    require_arg_count(args, 4, option)?;
    let db_path = PathBuf::from(&args[2]);
    let backup_path = PathBuf::from(&args[3]);
    WorkspaceBackupImporter::validate_path(&backup_path)?;
    let repository = Repository::open(&db_path)
        .with_context(|| format!("failed to open workspace database {}", db_path.display()))?;
    ensure_no_active_nodes_for_backup_import(&repository, &db_path)?;
    let import = WorkspaceBackupImporter::import_path(&repository, backup_path)?;
    Ok((db_path, import))
}

fn ensure_no_active_nodes_for_backup_import(repository: &Repository, db_path: &Path) -> Result<()> {
    if let Some(node) = repository
        .list_nodes()
        .with_context(|| format!("failed to inspect target workspace {}", db_path.display()))?
        .into_iter()
        .find(|node| node.status.is_active())
    {
        anyhow::bail!(
            "refusing to import backup into {} while node {} is {}; stop active nodes before importing",
            db_path.display(),
            node.name,
            node.status
        );
    }
    Ok(())
}
