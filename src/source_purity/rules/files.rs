use std::path::Path;

use crate::source_purity::SourcePurityFinding;

pub(super) fn disallowed_file_rule(name: &str, path: String) -> Option<SourcePurityFinding> {
    for rule in FILE_RULES {
        if (rule.matches)(name) {
            return Some(SourcePurityFinding {
                path,
                category: rule.category.to_string(),
                message: rule.message.to_string(),
            });
        }
    }
    None
}

struct FileRule {
    category: &'static str,
    message: &'static str,
    matches: fn(&str) -> bool,
}

const FILE_RULES: &[FileRule] = &[
    FileRule {
        category: "webview-application-config",
        message: "Tauri/WebView application config files are outside the pure native Rust application boundary",
        matches: is_webview_application_config,
    },
    FileRule {
        category: "node-toolchain-manifest",
        message: "Node package manifests and lockfiles are not part of the pure Rust application",
        matches: is_node_manifest,
    },
    FileRule {
        category: "frontend-tooling-config",
        message: "frontend build/test/lint configs should not re-enter the native Rust codebase",
        matches: is_frontend_config,
    },
    FileRule {
        category: "container-deployment-artifact",
        message: "container/server deployment files should not re-enter the native application source tree",
        matches: is_container_deployment_artifact,
    },
    FileRule {
        category: "web-server-deployment-artifact",
        message: "nginx or web-server setup artifacts belong to the old web deployment surface",
        matches: is_web_server_deployment_artifact,
    },
    FileRule {
        category: "frontend-source-file",
        message: "JavaScript, TypeScript, HTML, and CSS sources are outside the pure Rust application boundary",
        matches: is_frontend_source_extension,
    },
];

fn is_node_manifest(name: &str) -> bool {
    matches!(
        name,
        "package.json"
            | "package-lock.json"
            | "npm-shrinkwrap.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "bun.lock"
            | "bun.lockb"
    )
}

fn is_webview_application_config(name: &str) -> bool {
    matches!(
        name,
        "tauri.conf.json" | "tauri.conf.json5" | "tauri.conf.toml"
    )
}

fn is_frontend_config(name: &str) -> bool {
    (name.starts_with("tsconfig") && name.ends_with(".json"))
        || name.starts_with("vite.config.")
        || name.starts_with("vitest.config.")
        || name.starts_with("eslint.config.")
        || name.starts_with("postcss.config.")
        || name.starts_with("tailwind.config.")
}

fn is_container_deployment_artifact(name: &str) -> bool {
    matches!(
        name,
        "Dockerfile" | ".dockerignore" | "docker-compose.yml" | "docker-compose.yaml"
    )
}

fn is_web_server_deployment_artifact(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    (lower.contains("nginx") && lower.ends_with(".conf")) || lower == "setup-nginx.sh"
}

fn is_frontend_source_extension(name: &str) -> bool {
    let extension = Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    matches!(
        extension,
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "vue" | "svelte" | "html" | "css"
    )
}
