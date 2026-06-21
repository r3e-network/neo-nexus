use std::path::Path;

use crate::types::NodeType;

use super::super::probes::runtime_probe_args;
use super::fake::strings;

#[test]
fn neo_rs_probe_plan_covers_daemon_and_workspace_cli_help() {
    let probes = runtime_probe_args(NodeType::NeoRs, Path::new("neo-node"), &[]);

    assert!(probes.contains(&strings(&["--version"])));
    assert!(probes.contains(&strings(&["version"])));
    assert!(probes.contains(&strings(&["--help"])));
    assert!(probes.contains(&strings(&["node", "--help"])));
}

#[test]
fn neo_rs_wrapper_probe_preserves_runner_arguments() {
    let wrapper_args = strings(&["./target/release/neo-node"]);
    let probes = runtime_probe_args(NodeType::NeoRs, Path::new("/bin/sh"), &wrapper_args);

    assert!(probes.contains(&strings(&["./target/release/neo-node", "--version"])));
    assert!(probes.contains(&strings(&["./target/release/neo-node", "--help"])));
}
