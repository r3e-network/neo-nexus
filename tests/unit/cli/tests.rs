use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::{
    backup::{WorkspaceBackupExporter, WorkspaceBackupImporter},
    catalog::{PluginId, PluginState},
    config::ConfigGenerator,
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    release_pack::{ReleasePackagePlatform, ReleasePackager},
    repository::Repository,
    types::{Network, NewNode, NodeConfig, NodeStatus, NodeType, StorageEngine},
};

use super::{action_from_args, CliAction};

fn neo_rs_config_validation_node() -> NodeConfig {
    NodeConfig {
        id: "cli-neo-rs".to_string(),
        name: "cli neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 20332,
        p2p_port: 20333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

fn write_launch_pack_sidecar_manifest(root: &Path) -> Result<PathBuf> {
    let public_key = format!("02{}", "a".repeat(64));
    let manifest_path = root.join("manifest.json");
    std::fs::write(
        &manifest_path,
        format!(
            r#"{{
  "schema_version": 10,
  "generated_at_unix": 1800000000,
  "template": "Single validator",
  "runtime": "neo-rs",
  "network": "private",
  "network_magic": 1230301,
  "validators_count": 1,
  "seed_nodes": ["127.0.0.1:30333"],
  "committee": {{
"signer_count": 1,
"wallet_reference_count": 1,
"endpoint_reference_count": 1,
"sidecar_command_count": 1,
"public_keys": ["{public_key}"],
"secret_material_policy": "references-only-no-private-keys-or-passwords",
"preflight_policy": "check-native-wallet-paths-http-endpoints-and-sidecar-commands",
"signers": [
  {{
    "label": "committee-signer-1",
    "public_key": "{public_key}",
    "wallet_path": "wallets/validator-1.wallet.json",
    "signer_endpoint": "http://127.0.0.1:9021",
    "signer_command_template": "signer-bin/neo-signer --wallet {{wallet}} --listen {{endpoint}}",
    "signer_command": "signer-bin/neo-signer --wallet wallets/validator-1.wallet.json --listen http://127.0.0.1:9021",
    "signer_command_plan": {{
      "execution_policy": "argv-no-shell",
      "binary": "signer-bin/neo-signer",
      "arguments": [
        "--wallet",
        "wallets/validator-1.wallet.json",
        "--listen",
        "http://127.0.0.1:9021"
      ]
    }}
  }}
]
  }},
  "secret_provisioning": {{
"schema_version": 1,
"policy": "operator-provided-wallets-no-secret-material-in-launch-pack",
"wallet_provisioning_file": "wallet-provisioning.json",
"wallet_instructions_file": "wallets/README.md",
"recommended_wallet_root": "wallets",
"required_wallet_count": 1,
"wallet_reference_count": 1,
"missing_wallet_reference_count": 0,
"generated_secret_count": 0
  }},
  "scripts": {{
"runbook": "RUNBOOK.md",
"preflight_unix": "preflight-unix.sh",
"preflight_windows": "preflight-windows.ps1",
"health_unix": "health-unix.sh",
"health_windows": "health-windows.ps1",
"start_unix": "start-unix.sh",
"stop_unix": "stop-unix.sh",
"start_windows": "start-windows.ps1",
"stop_windows": "stop-windows.ps1"
  }},
  "artifacts": [],
  "nodes": []
}}"#
        ),
    )?;
    Ok(manifest_path)
}

fn write_fake_executable(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, b"#!/usr/bin/env sh\nexit 0\n")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions)?;
    }

    Ok(())
}

const VALID_NEP6_CONTRACT_PUBLIC_KEY: &str =
    "036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0";

fn valid_nep6_wallet_json() -> String {
    serde_json::json!({
        "name": "NeoNexus validator wallet",
        "version": "3.0",
        "scrypt": {
            "n": 16384,
            "r": 8,
            "p": 8
        },
        "accounts": [
            {
                "address": "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq",
                "label": "validator-1",
                "isDefault": true,
                "lock": false,
                "key": "6PYWB8m1bCnu5bQkRUKAwbZp2BHNvQ3BQRLbpLdTuizpyLkQPSZbtZfoxx",
                "contract": {
                    "script": "21036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0ac",
                    "parameters": [],
                    "deployed": false
                },
                "extra": null
            }
        ],
        "extra": null
    })
    .to_string()
}

mod alerts;
mod backup_wallet_launch;
mod basics;
mod config;
mod health;
mod quality;
mod reports;
