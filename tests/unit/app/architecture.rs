use std::path::{Path, PathBuf};

mod module_boundaries;
mod visual_contract;

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn rust_sources(root_file: &str, root_dir: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = vec![manifest_path(root_file)];
    let mut pending_dirs = vec![manifest_path(root_dir)];
    while let Some(dir) = pending_dirs.pop() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                pending_dirs.push(path);
            } else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs") {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

fn rust_sources_under(root_dir: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let mut pending_dirs = vec![manifest_path(root_dir)];
    while let Some(dir) = pending_dirs.pop() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                pending_dirs.push(path);
            } else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs") {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

fn assert_no_root_imports(source: &str, path: &Path, modules: &[&str]) {
    for module in modules {
        for pattern in [
            format!("use crate::{module}::"),
            format!("use crate::{{{module}::"),
            format!("use crate::{{\n    {module}::"),
            format!("crate::{module}::"),
        ] {
            assert!(
                !source.contains(&pattern),
                "{} should not import {module} directly from crate root",
                path.display()
            );
        }
    }
}

#[test]
fn native_app_entrypoint_uses_domain_binding_for_shared_core_services() -> anyhow::Result<()> {
    let app_source = std::fs::read_to_string(manifest_path("src/app.rs"))?;
    assert!(app_source.contains("mod domain;"));
    assert!(app_source.contains("use domain::*;"));
    assert!(!app_source.contains("use crate::core::{"));

    let domain_source = std::fs::read_to_string(manifest_path("src/app/domain.rs"))?;
    assert!(domain_source.contains("use crate::core::{"));
    for boundary in [
        "distribution::{",
        "node::{",
        "operations::{",
        "runtime::{",
        "security::{",
        "workspace::{",
    ] {
        assert!(domain_source.contains(boundary));
    }
    assert_no_root_imports(
        &domain_source,
        &manifest_path("src/app/domain.rs"),
        &[
            "alerts",
            "backup",
            "catalog",
            "config",
            "diagnostics",
            "event_journal_report",
            "events",
            "federation",
            "launch",
            "logs",
            "metrics",
            "plugins",
            "port_planner",
            "preflight",
            "private_network",
            "readiness_report",
            "release_pack",
            "repository",
            "roles",
            "rpc_health",
            "runtime",
            "runtime_smoke",
            "snapshots",
            "supervisor",
            "support_bundle",
            "types",
            "wallet",
            "watchdog",
            "workspace_integrity",
        ],
    );
    Ok(())
}

#[test]
fn operations_workspace_views_use_app_domain_binding() -> anyhow::Result<()> {
    for path in rust_sources("src/app/views/operations.rs", "src/app/views/operations")? {
        let source = std::fs::read_to_string(&path)?;
        if [
            "PluginState",
            "FleetDiagnostics",
            "PortMatrixRow",
            "ReadinessAction",
            "DiagnosticCheck",
            "NodeDiagnostics",
            "CheckSeverity",
            "RuntimeEventFilter",
            "EventSeverity",
            "WorkspaceBackupValidation",
        ]
        .iter()
        .any(|symbol| source.contains(symbol))
        {
            assert!(
                source.contains("domain::"),
                "{} should import shared Operations domain types through app::domain",
                path.display()
            );
        }
        assert_no_root_imports(
            &source,
            &path,
            &["backup", "catalog", "diagnostics", "events"],
        );
    }
    Ok(())
}

#[test]
fn runtime_manager_views_use_app_domain_binding() -> anyhow::Result<()> {
    for path in rust_sources("src/app/views/runtimes.rs", "src/app/views/runtimes")? {
        let source = std::fs::read_to_string(&path)?;
        if [
            "RuntimeInstallation",
            "RuntimePlatform",
            "RuntimeCatalogUpgradePlan",
            "NodeConfig",
            "NodeType",
            "format_bytes",
        ]
        .iter()
        .any(|symbol| source.contains(symbol))
        {
            assert!(
                source.contains("domain::"),
                "{} should import shared Runtime Manager domain types through app::domain",
                path.display()
            );
        }
        assert_no_root_imports(&source, &path, &["metrics", "runtime", "types"]);
    }
    Ok(())
}
