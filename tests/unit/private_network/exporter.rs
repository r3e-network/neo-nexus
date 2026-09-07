//! Tests for private network deployment export metadata

use std::path::PathBuf;

use crate::private_network::PrivateNetworkDeploymentExport;

fn sample_export() -> PrivateNetworkDeploymentExport {
    let root = PathBuf::from("/tmp/launch-pack");
    PrivateNetworkDeploymentExport {
        root_path: root.clone(),
        manifest_path: root.join("manifest.json"),
        start_order_path: root.join("start-order.txt"),
        runbook_path: root.join("runbook.md"),
        wallet_provisioning_path: root.join("wallets.json"),
        wallet_instructions_path: root.join("wallets.md"),
        preflight_unix_path: root.join("preflight.sh"),
        preflight_windows_path: root.join("preflight.ps1"),
        health_unix_path: root.join("health.sh"),
        health_windows_path: root.join("health.ps1"),
        start_unix_path: root.join("start.sh"),
        stop_unix_path: root.join("stop.sh"),
        start_windows_path: root.join("start.ps1"),
        stop_windows_path: root.join("stop.ps1"),
        node_count: 4,
        config_count: 4,
        network_magic: 0x334F454E,
        validators_count: 4,
        bytes_written: 2048,
    }
}

#[test]
fn deployment_export_retains_all_artifact_paths_under_root() {
    let export = sample_export();
    for path in [
        &export.manifest_path,
        &export.start_order_path,
        &export.runbook_path,
        &export.preflight_unix_path,
        &export.start_windows_path,
        &export.stop_windows_path,
    ] {
        assert!(
            path.starts_with(&export.root_path),
            "artifact {} should live under the launch-pack root",
            path.display()
        );
    }
}

#[test]
fn deployment_export_counts_and_magic_are_preserved() {
    let export = sample_export();
    assert_eq!(export.node_count, 4);
    assert_eq!(export.config_count, 4);
    assert_eq!(export.validators_count, 4);
    assert_eq!(export.network_magic, 0x334F454E);
    assert_eq!(export.bytes_written, 2048);
}

#[test]
fn deployment_export_equality_is_structural() {
    assert_eq!(sample_export(), sample_export());
    let mut mutated = sample_export();
    mutated.bytes_written += 1;
    assert_ne!(sample_export(), mutated);
}
