use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupportBundleLogDiagnosisReport {
    pub schema_version: u32,
    pub log_dir: String,
    pub node_count: usize,
    pub warning_count: usize,
    pub critical_count: usize,
    pub entries: Vec<SupportBundleLogDiagnosis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupportBundleLogDiagnosis {
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub log_path: String,
    pub exists: bool,
    pub bytes: u64,
    pub truncated: bool,
    pub status: String,
    pub summary: String,
    pub finding_count: usize,
    pub findings: Vec<SupportBundleLogFinding>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupportBundleLogFinding {
    pub label: String,
    pub line_number: usize,
    pub excerpt: String,
    pub recommendation: String,
    pub status: String,
}
