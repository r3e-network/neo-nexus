use std::path::PathBuf;

use crate::types::NodeType;

use super::*;

#[test]
fn runtime_installation_filter_matches_runtime_trust_platform_and_query() {
    let platform = RuntimePlatform::current();
    let other = RuntimePlatform {
        os: "other".to_string(),
        arch: "arch".to_string(),
    };
    let installations = [
        installation("neo-rs-current", NodeType::NeoRs, platform.clone(), true),
        installation("neo-go-other", NodeType::NeoGo, other, false),
    ];
    let filter =
        RuntimeInstallationFilter::new(Some(NodeType::NeoRs), Some(true), Some(true), "current");

    let filtered = filter_runtime_installations(&installations, &platform, &filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].package_id, "neo-rs-current");
}

#[test]
fn runtime_release_filter_matches_runtime_platform_and_query() {
    let platform = RuntimePlatform::current();
    let releases = [
        release("neo-rs-v2", NodeType::NeoRs, platform.clone()),
        release("neo-go-v1", NodeType::NeoGo, other_platform()),
    ];
    let filter = RuntimeReleaseFilter::new(Some(NodeType::NeoRs), Some(true), "neo-node");

    let filtered = filter_runtime_releases(&releases, &platform, &filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "neo-rs-v2");
}

fn installation(
    id: &str,
    node_type: NodeType,
    platform: RuntimePlatform,
    signed: bool,
) -> RuntimeInstallation {
    RuntimeInstallation {
        package_id: id.to_string(),
        label: format!("{id} package"),
        node_type,
        version: "v1.0.0".to_string(),
        platform,
        binary_path: PathBuf::from(format!("/runtimes/{id}/neo-node")),
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
        url: format!("https://downloads.example.com/{id}/neo-node"),
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
