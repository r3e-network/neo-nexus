use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceReadinessReport {
    pub schema_version: u32,
    pub application: &'static str,
    pub application_version: String,
    pub generated_at_unix: u64,
    pub database: String,
    pub status: &'static str,
    pub score: usize,
    pub node_count: usize,
    pub ready_nodes: usize,
    pub critical_count: usize,
    pub warning_count: usize,
    pub findings: Vec<WorkspaceReadinessFindingReport>,
    pub nodes: Vec<WorkspaceReadinessNodeReport>,
}

impl WorkspaceReadinessReport {
    pub fn exit_code(&self) -> i32 {
        if self.critical_count == 0 {
            0
        } else {
            1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceReadinessFindingReport {
    pub severity: &'static str,
    pub node_id: String,
    pub node_name: String,
    pub title: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceReadinessNodeReport {
    pub node_id: String,
    pub node_name: String,
    pub score: usize,
    pub status: &'static str,
    pub critical_count: usize,
    pub warning_count: usize,
    pub checks: Vec<WorkspaceReadinessCheckReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceReadinessCheckReport {
    pub severity: &'static str,
    pub title: &'static str,
    pub detail: String,
}
