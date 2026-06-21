use super::*;

#[test]
fn runtime_filters_limit_inventory_and_catalog_selection() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let platform = RuntimePlatform::current();
    repository.upsert_runtime_installation(&installation(
        "neo-rs-current",
        NodeType::NeoRs,
        platform.clone(),
        true,
    ))?;
    repository.upsert_runtime_installation(&installation(
        "neo-go-other",
        NodeType::NeoGo,
        other_platform(),
        false,
    ))?;
    let mut app = NeoNexusApp::new(repository);

    app.runtime_inventory_query = "neo-rs".to_string();
    app.runtime_inventory_type_filter = Some(NodeType::NeoRs);
    app.runtime_inventory_signed_filter = Some(true);
    app.runtime_inventory_platform_filter = Some(true);
    let installations = app.runtime_installations();
    let visible_installations = app.filtered_runtime_installations(&installations);
    assert_eq!(visible_installations.len(), 1);
    assert_eq!(visible_installations[0].package_id, "neo-rs-current");

    app.selected_runtime_installation = Some("neo-go-other".to_string());
    app.runtime_page = 5;
    app.ensure_valid_runtime_selection(&installations);
    assert_eq!(
        app.selected_runtime_installation.as_deref(),
        Some("neo-rs-current")
    );
    assert_eq!(app.runtime_page, 0);

    app.runtime_catalog = Some(RuntimeReleaseCatalog {
        schema_version: 1,
        generated_at_unix: Some(1_800_000_000),
        releases: vec![
            release("neo-rs-release", NodeType::NeoRs, platform.clone()),
            release("neo-go-release", NodeType::NeoGo, other_platform()),
        ],
    });
    app.runtime_catalog_query = "neo-rs".to_string();
    app.runtime_catalog_type_filter = Some(NodeType::NeoRs);
    app.runtime_catalog_platform_filter = Some(true);
    app.selected_runtime_release = Some("neo-go-release".to_string());
    app.runtime_catalog_page = 4;
    app.ensure_valid_runtime_release_selection();

    assert_eq!(
        app.selected_runtime_release.as_deref(),
        Some("neo-rs-release")
    );
    assert_eq!(app.runtime_catalog_page, 0);

    Ok(())
}

fn installation(
    package_id: &str,
    node_type: NodeType,
    platform: RuntimePlatform,
    signed: bool,
) -> RuntimeInstallation {
    RuntimeInstallation {
        package_id: package_id.to_string(),
        label: format!("{package_id} package"),
        node_type,
        version: "v1.0.0".to_string(),
        platform,
        binary_path: PathBuf::from(format!("/runtimes/{package_id}/neo-node")),
        sha256: "a".repeat(64),
        signature_verified: signed,
        signer_public_key: signed.then(|| "trusted-key".to_string()),
        bytes: 1024,
        installed_at_unix: 1_800_000_000,
    }
}

fn release(id: &str, node_type: NodeType, platform: RuntimePlatform) -> RuntimeRelease {
    RuntimeRelease {
        id: id.to_string(),
        label: format!("{id} release"),
        node_type,
        version: "v1.0.0".to_string(),
        platform,
        url: format!("https://downloads.example.com/{id}/neo-node.zip"),
        file_name: "neo-node.zip".to_string(),
        executable_name: "neo-node".to_string(),
        expected_sha256: "b".repeat(64),
        max_bytes: 2048,
    }
}

fn other_platform() -> RuntimePlatform {
    RuntimePlatform {
        os: "other".to_string(),
        arch: "arch".to_string(),
    }
}
