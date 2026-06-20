use crate::source_purity::SourcePurityFinding;

pub(super) fn should_skip_directory_rule(name: &str) -> bool {
    matches!(name, ".git" | "target" | "dist")
}

pub(super) fn disallowed_directory_rule(name: &str, path: String) -> Option<SourcePurityFinding> {
    let (category, message) = match name {
        "node_modules" => (
            "node-dependency-directory",
            "Node dependencies do not belong in the pure Rust source tree",
        ),
        "web" | "frontend" | ".next" | "coverage" => (
            "frontend-directory",
            "frontend/web application directories are outside the native Rust application boundary",
        ),
        "src-tauri" => (
            "webview-application-directory",
            "Tauri/WebView application directories are outside the native Rust application boundary",
        ),
        _ => return None,
    };
    Some(SourcePurityFinding {
        path,
        category: category.to_string(),
        message: message.to_string(),
    })
}
