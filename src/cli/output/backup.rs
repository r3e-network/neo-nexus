use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::core::workspace::{
    WorkspaceBackupExport, WorkspaceBackupImport, WorkspaceBackupValidation,
};

use super::json_text;

#[derive(Debug, Serialize)]
struct BackupValidationJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    validation: &'a WorkspaceBackupValidation,
}

#[derive(Debug, Serialize)]
struct BackupExportJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    export: &'a WorkspaceBackupExport,
}

#[derive(Debug, Serialize)]
struct BackupImportJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    target_database: String,
    import: &'a WorkspaceBackupImport,
}

pub(in crate::cli) fn backup_validation_json_text(
    validation: &WorkspaceBackupValidation,
) -> Result<String> {
    json_text(&BackupValidationJsonReport {
        schema_version: 1,
        status: "ok",
        validation,
    })
}

pub(in crate::cli) fn backup_export_json_text(export: &WorkspaceBackupExport) -> Result<String> {
    json_text(&BackupExportJsonReport {
        schema_version: 1,
        status: "ok",
        export,
    })
}

pub(in crate::cli) fn backup_import_json_text(
    db_path: &Path,
    import: &WorkspaceBackupImport,
) -> Result<String> {
    json_text(&BackupImportJsonReport {
        schema_version: 1,
        status: "ok",
        target_database: db_path.display().to_string(),
        import,
    })
}
