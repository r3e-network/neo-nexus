use std::{fs, path::Path, path::PathBuf};

use anyhow::Result;

use super::*;
use crate::types::{Network, NodeConfig, NodeStatus, NodeType};

fn node(path: PathBuf, node_type: NodeType, args: Vec<String>) -> NodeConfig {
    NodeConfig {
        id: "node".to_string(),
        name: "node".to_string(),
        node_type,
        network: Network::Testnet,
        binary_path: path,
        args,
        runtime_version: "v1".to_string(),
        storage_engine: node_type.default_storage_engine(),
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
        status: NodeStatus::Stopped,
        pid: None,
    }
}

fn write_binary(path: &Path) -> Result<()> {
    fs::write(path, b"#!/usr/bin/env sh\nexit 0\n")?;
    make_executable(path)?;
    Ok(())
}

fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

#[test]
fn preflight_accepts_matching_neo_go_executable() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary = temp_dir.path().join(if cfg!(windows) {
        "neo-go.exe"
    } else {
        "neo-go"
    });
    write_binary(&binary)?;

    let report = inspect_node_binary(&node(binary, NodeType::NeoGo, Vec::new()));

    assert!(!report.has_blockers());
    assert_eq!(report.status_label(), "ready");
    assert!(report.checks.iter().any(|check| {
        check.title == "Runtime identity" && check.severity == PreflightSeverity::Pass
    }));
    Ok(())
}

#[test]
fn preflight_blocks_missing_binary() {
    let report = inspect_node_binary(&node(
        PathBuf::from("/definitely/missing/neo-go"),
        NodeType::NeoGo,
        Vec::new(),
    ));

    assert!(report.has_blockers());
    assert_eq!(report.status_label(), "blocked");
}

#[test]
fn preflight_accepts_dotnet_neo_cli_wrapper() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let dotnet = temp_dir.path().join(if cfg!(windows) {
        "dotnet.exe"
    } else {
        "dotnet"
    });
    write_binary(&dotnet)?;
    let report = inspect_node_binary(&node(
        dotnet,
        NodeType::NeoCli,
        vec!["Neo.CLI.dll".to_string()],
    ));

    assert!(!report.has_blockers());
    assert!(report.checks.iter().any(|check| {
        check.title == "Runtime identity" && check.severity == PreflightSeverity::Pass
    }));
    Ok(())
}

#[test]
fn preflight_warns_for_unexpected_runtime_name() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary = temp_dir.path().join(if cfg!(windows) {
        "custom-node.exe"
    } else {
        "custom-node"
    });
    write_binary(&binary)?;

    let report = inspect_node_binary(&node(binary, NodeType::NeoRs, Vec::new()));

    assert!(!report.has_blockers());
    assert_eq!(report.status_label(), "review");
    assert!(report.checks.iter().any(|check| {
        check.title == "Runtime identity" && check.severity == PreflightSeverity::Warning
    }));
    Ok(())
}
