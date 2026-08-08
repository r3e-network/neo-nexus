//! A node has to boot with the duty the workspace says it performs.
//!
//! Applying a duty writes the config correctly. Then Start rewrote that same
//! file with a context-free render and launched a plain relay — while the
//! workbench went on showing the duty, because the duty *was* recorded. It just
//! never reached the file the node booted from, so a consensus node relayed and
//! nothing said so.
//!
//! Both entry points did it: the GUI's start/restart and the CLI's
//! `--node-start` / `--node-restart`, since all of them route through
//! `execute_node_launch`.

use crate::*;

/// A duty applied in the workspace must survive the launch that rewrites the
/// config. Asserted on the file the node would actually read.
#[test]
fn a_launch_writes_the_config_for_the_recorded_duty() {
    let repo = create_repo();
    let node_id = create_node(&repo, "consensus-node", NodeType::NeoGo);
    repo.set_node_role(&node_id, Some(NodeRole::Consensus))
        .unwrap();
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("neo-go.yml");
    let plan = LaunchPlanner::plan(&node, &config_path, temp.path());
    let mut supervisor = ProcessSupervisor::default();

    // The binary does not exist, so supervision fails — but the config is
    // written first, which is the step under test.
    let _ = execute_node_launch(
        &repo,
        &mut supervisor,
        &node,
        &plan,
        temp.path().join("node.log"),
        LaunchAction::Start,
        Some(ManagedConfig {
            path: &config_path,
            plugins: &[],
        }),
    );

    let written = std::fs::read_to_string(&config_path).expect("the managed config was written");
    assert!(
        written.contains("Consensus:"),
        "a node recorded as Consensus launched with a relaying config:\n{written}",
    );
}

/// Restart takes the same path, so it must not undo what Start got right.
#[test]
fn a_restart_also_writes_the_config_for_the_recorded_duty() {
    let repo = create_repo();
    let node_id = create_node(&repo, "oracle-node", NodeType::NeoGo);
    repo.set_node_role(&node_id, Some(NodeRole::Oracle))
        .unwrap();
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("neo-go.yml");
    let plan = LaunchPlanner::plan(&node, &config_path, temp.path());
    let mut supervisor = ProcessSupervisor::default();

    let _ = execute_node_launch(
        &repo,
        &mut supervisor,
        &node,
        &plan,
        temp.path().join("node.log"),
        LaunchAction::Restart,
        Some(ManagedConfig {
            path: &config_path,
            plugins: &[],
        }),
    );

    let written = std::fs::read_to_string(&config_path).expect("the managed config was written");
    assert!(written.contains("Oracle:"), "{written}");
}

/// A node with no duty still gets a plain relaying config — the fix must not
/// invent a duty for nodes that have none.
#[test]
fn a_node_without_a_duty_still_launches_as_a_relay() {
    let repo = create_repo();
    let node_id = create_node(&repo, "plain-node", NodeType::NeoGo);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("neo-go.yml");
    let plan = LaunchPlanner::plan(&node, &config_path, temp.path());
    let mut supervisor = ProcessSupervisor::default();

    let _ = execute_node_launch(
        &repo,
        &mut supervisor,
        &node,
        &plan,
        temp.path().join("node.log"),
        LaunchAction::Start,
        Some(ManagedConfig {
            path: &config_path,
            plugins: &[],
        }),
    );

    let written = std::fs::read_to_string(&config_path).expect("written");
    for signing in ["Consensus:", "Oracle:", "StateRoot:", "P2PNotary:"] {
        assert!(
            !written.contains(signing),
            "a node with no duty got a {signing} section:\n{written}",
        );
    }
}
