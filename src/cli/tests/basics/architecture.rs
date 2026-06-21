use std::path::{Path, PathBuf};

const SHARED_DOMAIN_MODULES: &[&str] = &[
    "alerts",
    "backup",
    "catalog",
    "ci_policy",
    "config",
    "diagnostics",
    "events",
    "federation",
    "logs",
    "metrics",
    "native_ui",
    "plugins",
    "private_network",
    "readiness_report",
    "release_pack",
    "repository",
    "roles",
    "runtime",
    "runtime_smoke",
    "snapshots",
    "source_purity",
    "source_quality",
    "support_bundle",
    "types",
    "wallet",
    "workspace_integrity",
];

#[test]
fn cli_production_modules_use_core_facade_for_shared_domain_services() -> anyhow::Result<()> {
    for path in cli_production_sources()? {
        let source = std::fs::read_to_string(&path)?;
        assert_no_root_imports(&source, &path);
    }
    Ok(())
}

fn cli_production_sources() -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = vec![manifest_path("src/cli.rs")];
    let mut pending_dirs = vec![manifest_path("src/cli")];
    while let Some(dir) = pending_dirs.pop() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                pending_dirs.push(path);
            } else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs")
                && !is_cli_test_source(&path)?
            {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn is_cli_test_source(path: &Path) -> anyhow::Result<bool> {
    Ok(path
        .strip_prefix(manifest_path(""))?
        .display()
        .to_string()
        .starts_with("src/cli/tests/"))
}

fn assert_no_root_imports(source: &str, path: &Path) {
    for module in SHARED_DOMAIN_MODULES {
        for pattern in [
            format!("use crate::{module}::"),
            format!("use crate::{{{module}::"),
            format!("use crate::{{\n    {module}::"),
            format!("crate::{module}::"),
        ] {
            assert!(
                !source.contains(&pattern),
                "{} should import {module} through src/core/, not directly from crate root",
                path.display()
            );
        }
    }
}
