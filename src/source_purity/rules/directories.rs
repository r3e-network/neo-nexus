use crate::source_purity::SourcePurityFinding;

pub(super) fn should_skip_directory_rule(name: &str) -> bool {
    matches!(name, ".git" | "target" | "dist")
}

pub(super) fn disallowed_directory_rule(name: &str, path: String) -> Option<SourcePurityFinding> {
    // `web` is deliberately absent: since 4.0 the workbench is a web service,
    // and `src/web` is plain Rust server code. The invariant this gate protects
    // is "no frontend toolchain or assets in the tree", which the per-file
    // `frontend-source-file` rule enforces on .js/.ts/.tsx/.html/.css wherever
    // they appear — a directory name proves nothing either way.
    let (category, message) = match name {
        "node_modules" => (
            "node-dependency-directory",
            "Node dependencies do not belong in the pure Rust source tree",
        ),
        "frontend" | ".next" | "coverage" => (
            "frontend-directory",
            "frontend application directories are outside the native Rust application boundary",
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
