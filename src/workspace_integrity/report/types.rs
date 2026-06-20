use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceIntegrityReport {
    pub schema_version: u32,
    pub status: String,
    pub application: &'static str,
    pub application_version: String,
    pub checked_at_unix: u64,
    pub database: String,
    pub database_bytes: u64,
    pub sqlite_user_version: u32,
    pub sqlite_page_size: u64,
    pub sqlite_page_count: u64,
    pub sqlite_freelist_count: u64,
    pub integrity_check: Vec<String>,
    pub foreign_key_violations: Vec<ForeignKeyViolation>,
    pub required_tables: Vec<RequiredTableCheck>,
    pub required_indexes: Vec<RequiredIndexCheck>,
    pub row_counts: Vec<TableRowCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForeignKeyViolation {
    pub table: String,
    pub row_id: Option<i64>,
    pub parent_table: String,
    pub foreign_key_index: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequiredTableCheck {
    pub table: String,
    pub present: bool,
    pub column_count: usize,
    pub expected_column_count: usize,
    pub missing_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequiredIndexCheck {
    pub table: String,
    pub index: String,
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableRowCount {
    pub table: String,
    pub rows: u64,
}
