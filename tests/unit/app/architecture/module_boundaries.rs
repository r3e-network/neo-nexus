use super::*;

const SHARED_DOMAIN_MODULES: &[&str] = &[
    "alerts",
    "backup",
    "catalog",
    "config",
    "diagnostics",
    "events",
    "federation",
    "logs",
    "metrics",
    "plugins",
    "private_network",
    "repository",
    "roles",
    "runtime",
    "snapshots",
    "types",
    "wallet",
    "workspace_integrity",
];

#[test]
fn native_views_use_app_domain_for_shared_domain_modules() -> anyhow::Result<()> {
    for path in rust_sources("src/app/views.rs", "src/app/views")? {
        let source = std::fs::read_to_string(&path)?;
        assert_no_root_imports(&source, &path, SHARED_DOMAIN_MODULES);
    }
    Ok(())
}

#[test]
fn native_app_support_modules_use_app_domain_for_shared_domain_modules() -> anyhow::Result<()> {
    for path in rust_sources_under("src/app")? {
        let relative = path.strip_prefix(manifest_path(""))?;
        let relative_text = relative.display().to_string();
        if is_exempt_app_source(&relative_text) {
            continue;
        }

        let source = std::fs::read_to_string(&path)?;
        assert_no_root_imports(&source, &path, SHARED_DOMAIN_MODULES);
    }
    Ok(())
}

fn is_exempt_app_source(relative_text: &str) -> bool {
    relative_text == "src/app/domain.rs"
        || relative_text == "src/app/tests.rs"
        || relative_text.starts_with("src/app/views/")
        || relative_text.starts_with("src/app/tests/")
        || relative_text.ends_with("/tests.rs")
}

/// A view must read a node's RPC health through the high-level core operation
/// (`node_rpc_health_history` / `latest_node_rpc_health`), not by calling the
/// repository's row API during paint. Scanning the raw `list_rpc_health` /
/// `latest_rpc_health` repository calls in view source keeps the persistence
/// layer behind the core facade and prevents a SQLite query per frame.
#[test]
fn views_read_rpc_health_through_core_not_the_repository() -> anyhow::Result<()> {
    for path in rust_sources("src/app/views.rs", "src/app/views")? {
        let source = std::fs::read_to_string(&path)?;
        for forbidden in [
            ".repository.list_rpc_health",
            ".repository.latest_rpc_health",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} reaches into the repository with {forbidden}; read RPC health through the core::node_health operation instead",
                path.display(),
            );
        }
    }
    Ok(())
}
