#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckSeverity {
    Pass,
    Info,
    Warning,
    Critical,
}

impl CheckSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }

    pub(in crate::diagnostics) fn score_penalty(self) -> usize {
        match self {
            Self::Pass | Self::Info => 0,
            Self::Warning => 15,
            Self::Critical => 35,
        }
    }
}
