use anyhow::Result;

use super::types::WorkspaceIntegrityReport;

impl WorkspaceIntegrityReport {
    pub fn to_cli_text(&self) -> String {
        let mut lines = self.summary_lines();
        append_integrity_checks(&mut lines, self);
        append_table_checks(&mut lines, self);
        append_index_checks(&mut lines, self);
        append_foreign_key_violations(&mut lines, self);
        append_row_counts(&mut lines, self);
        lines.push(String::new());
        lines.join("\n")
    }

    pub fn to_json_text(&self) -> Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(self)?))
    }

    fn summary_lines(&self) -> Vec<String> {
        let present_tables = self
            .required_tables
            .iter()
            .filter(|table| table.present)
            .count();
        let present_indexes = self
            .required_indexes
            .iter()
            .filter(|index| index.present)
            .count();

        vec![
            format!("workspace-integrity: {}", self.status_label()),
            format!("database: {}", self.database),
            format!("database-bytes: {}", self.database_bytes),
            format!("sqlite-user-version: {}", self.sqlite_user_version),
            format!("sqlite-page-size: {}", self.sqlite_page_size),
            format!("sqlite-page-count: {}", self.sqlite_page_count),
            format!("sqlite-freelist-count: {}", self.sqlite_freelist_count),
            format!("tables: {present_tables}/{}", self.required_tables.len()),
            format!("indexes: {present_indexes}/{}", self.required_indexes.len()),
            format!(
                "foreign-key-violations: {}",
                self.foreign_key_violations.len()
            ),
        ]
    }
}

fn append_integrity_checks(lines: &mut Vec<String>, report: &WorkspaceIntegrityReport) {
    for message in &report.integrity_check {
        lines.push(format!("integrity-check: {message}"));
    }
}

fn append_table_checks(lines: &mut Vec<String>, report: &WorkspaceIntegrityReport) {
    for table in &report.required_tables {
        lines.push(format!(
            "table: {} | {} | columns {}/{}",
            table.table,
            if table.present { "present" } else { "missing" },
            table.column_count,
            table.expected_column_count
        ));
        for column in &table.missing_columns {
            lines.push(format!("missing-column: {}.{}", table.table, column));
        }
    }
}

fn append_index_checks(lines: &mut Vec<String>, report: &WorkspaceIntegrityReport) {
    for index in &report.required_indexes {
        lines.push(format!(
            "index: {} | {} | {}",
            index.index,
            index.table,
            if index.present { "present" } else { "missing" }
        ));
    }
}

fn append_foreign_key_violations(lines: &mut Vec<String>, report: &WorkspaceIntegrityReport) {
    if report.foreign_key_violations.is_empty() {
        lines.push("foreign-key: none".to_string());
        return;
    }

    for violation in &report.foreign_key_violations {
        lines.push(format!(
            "foreign-key: {} row {:?} references {} via {}",
            violation.table, violation.row_id, violation.parent_table, violation.foreign_key_index
        ));
    }
}

fn append_row_counts(lines: &mut Vec<String>, report: &WorkspaceIntegrityReport) {
    for row_count in &report.row_counts {
        lines.push(format!("rows: {} | {}", row_count.table, row_count.rows));
    }
}
