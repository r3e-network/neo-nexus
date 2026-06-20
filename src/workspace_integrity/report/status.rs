use super::types::WorkspaceIntegrityReport;

impl WorkspaceIntegrityReport {
    pub fn is_success(&self) -> bool {
        self.integrity_check.len() == 1
            && self.integrity_check[0] == "ok"
            && self.foreign_key_violations.is_empty()
            && self
                .required_tables
                .iter()
                .all(|table| table.present && table.missing_columns.is_empty())
            && self.required_indexes.iter().all(|index| index.present)
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_success() {
            "ok"
        } else {
            "failed"
        }
    }

    pub fn exit_code(&self) -> i32 {
        if self.is_success() {
            0
        } else {
            1
        }
    }
}
