use crate::logs::LogDiagnosisStatus;

use super::model::{LogFailureRule, LogRuleMatcher};

const PERMISSION_MARKERS: &[&str] = &[
    "permission denied",
    "operation not permitted",
    "access is denied",
    "not executable",
];

const MISSING_FILE_MARKERS: &[&str] = &[
    "no such file or directory",
    "cannot find the path",
    "file not found",
    "not found",
];

const CONFIG_PARSE_MARKERS: &[&str] = &[
    "failed to parse",
    "parse error",
    "invalid config",
    "configuration error",
    "toml",
    "yaml",
    "json parse",
];

const RUNTIME_CRASH_MARKERS: &[&str] = &[
    "panic",
    "fatal",
    "unhandled exception",
    "segmentation fault",
];

const RUNTIME_ERROR_MARKERS: &[&str] = &[" error ", "error:", "[error]", "failed"];

const RUNTIME_WARNING_MARKERS: &[&str] = &[" warning ", "warning:", "[warn]", " warn "];

pub(super) fn known_failure_rules() -> &'static [LogFailureRule] {
    &KNOWN_FAILURE_RULES
}

const KNOWN_FAILURE_RULES: [LogFailureRule; 8] = [
    LogFailureRule {
        label: "Port binding failure",
        recommendation: "Use Node Studio Fix Ports, then restart the stopped node.",
        status: LogDiagnosisStatus::Critical,
        matcher: LogRuleMatcher::PortBinding,
    },
    LogFailureRule {
        label: "Permission failure",
        recommendation:
            "Check binary permissions, working directory access, and platform security prompts.",
        status: LogDiagnosisStatus::Critical,
        matcher: LogRuleMatcher::ContainsAny(PERMISSION_MARKERS),
    },
    LogFailureRule {
        label: "Missing file or command",
        recommendation: "Probe the binary path and verify referenced config/data paths exist.",
        status: LogDiagnosisStatus::Critical,
        matcher: LogRuleMatcher::ContainsAny(MISSING_FILE_MARKERS),
    },
    LogFailureRule {
        label: "Configuration parse failure",
        recommendation:
            "Re-export managed config and compare runtime arguments for custom config overrides.",
        status: LogDiagnosisStatus::Critical,
        matcher: LogRuleMatcher::ContainsAny(CONFIG_PARSE_MARKERS),
    },
    LogFailureRule {
        label: "Database lock",
        recommendation:
            "Stop duplicate node processes and confirm the managed data directory is not shared.",
        status: LogDiagnosisStatus::Critical,
        matcher: LogRuleMatcher::DatabaseLock,
    },
    LogFailureRule {
        label: "Runtime crash",
        recommendation:
            "Open the latest log tail, run Smoke Runtime, and verify runtime version compatibility.",
        status: LogDiagnosisStatus::Critical,
        matcher: LogRuleMatcher::ContainsAny(RUNTIME_CRASH_MARKERS),
    },
    LogFailureRule {
        label: "Runtime error",
        recommendation:
            "Inspect the surrounding log lines and run Operations readiness before retrying.",
        status: LogDiagnosisStatus::Warning,
        matcher: LogRuleMatcher::ContainsAny(RUNTIME_ERROR_MARKERS),
    },
    LogFailureRule {
        label: "Runtime warning",
        recommendation: "Review warning context and confirm RPC Health after startup.",
        status: LogDiagnosisStatus::Warning,
        matcher: LogRuleMatcher::ContainsAny(RUNTIME_WARNING_MARKERS),
    },
];
