use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogSnapshot {
    pub path: PathBuf,
    pub exists: bool,
    pub bytes: u64,
    pub truncated: bool,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub number: usize,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogDiagnosisStatus {
    NoLog,
    Quiet,
    Informational,
    Warning,
    Critical,
}

impl LogDiagnosisStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NoLog => "no-log",
            Self::Quiet => "quiet",
            Self::Informational => "informational",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }

    pub(super) fn rank(self) -> u8 {
        match self {
            Self::NoLog => 0,
            Self::Quiet => 1,
            Self::Informational => 2,
            Self::Warning => 3,
            Self::Critical => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFinding {
    pub label: String,
    pub line_number: usize,
    pub excerpt: String,
    pub recommendation: String,
    pub status: LogDiagnosisStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogDiagnosis {
    pub status: LogDiagnosisStatus,
    pub summary: String,
    pub findings: Vec<LogFinding>,
    pub recommendations: Vec<String>,
}
