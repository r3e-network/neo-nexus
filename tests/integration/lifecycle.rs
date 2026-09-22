//! The node lifecycle, end to end, for every node type NeoNexus supervises.
//!
//! Each test drives the core the web workbench and the headless CLI share —
//! `execute_node_launch` and `stop_node_runtime` over a real SQLite workspace,
//! with the real launch planner and config exporter and a `ProcessSupervisor`
//! holding a real child process — and checks the three places a launch leaves
//! evidence: the process table, the workspace row, and the node's log.
//!
//! The runtime is the stand-in from `tests/support/stub_runtime.rs`. It ignores
//! each client's flags and stays running, so what is under test is NeoNexus's
//! lifecycle, not a Neo client.

use std::collections::BTreeSet;

use anyhow::{bail, Result};
use neo_nexus::{
    config::ConfigFormat,
    node_lifecycle::{
        execute_node_launch, stop_node_runtime, LaunchAction, ManagedConfig, NodeLaunchOutcome,
        NodeLaunchRequest,
    },
    supervisor::{process_is_live, PidStop, ProcessStop, ProcessSupervisor},
    types::{NodeConfig, NodeStatus, NodeType},
};

use super::common::{stub_runtime_binary, LaunchInputs, Workspace};

/// Launch `node` through the shared core, rendering its managed config first,
/// as `--node-start` and the workbench's Start button do.
fn launch(
    workspace: &Workspace,
    supervisor: &mut ProcessSupervisor,
    node: &NodeConfig,
    action: LaunchAction,
) -> Result<(NodeLaunchOutcome, LaunchInputs)> {
    let inputs = workspace.launch_inputs(node)?;
    let outcome = execute_node_launch(
        workspace.repository(),
        supervisor,
        NodeLaunchRequest {
            signer_registry: None,
            node,
            plan: &inputs.plan,
            log_path: inputs.log_path.clone(),
            action,
            managed_config: Some(ManagedConfig {
                path: &inputs.config_path,
                plugins: &[],
            }),
        },
    );
    Ok((outcome, inputs))
}

fn started_pid(outcome: NodeLaunchOutcome) -> Result<u32> {
    match outcome {
        NodeLaunchOutcome::Started { pid, .. } => Ok(pid),
        NodeLaunchOutcome::Failed { message } => bail!("the launch failed: {message}"),
    }
}

/// Stop a node through the shared stop protocol and return what it stopped.
fn stop(
    workspace: &Workspace,
    supervisor: &mut ProcessSupervisor,
    node_id: &str,
) -> Result<ProcessStop> {
    let node = workspace.stored(node_id)?;
    let log_path = workspace.launch_inputs(&node)?.log_path;
    match stop_node_runtime(workspace.repository(), supervisor, &node, log_path)? {
        PidStop::Stopped(stopped) => Ok(stopped),
        other => bail!("{} was not stopped: {other:?}", node.name),
    }
}

fn parses_as(format: ConfigFormat, text: &str) -> Result<()> {
    match format {
        ConfigFormat::Json => serde_json::from_str::<serde_json::Value>(text).map(|_| ())?,
        ConfigFormat::Yaml => serde_yaml::from_str::<serde_yaml::Value>(text).map(|_| ())?,
        ConfigFormat::Toml => toml::from_str::<toml::Value>(text).map(|_| ())?,
    }
    Ok(())
}

/// Start one node of `node_type`, check what the start left behind, stop it,
/// and check what the stop left behind.
fn launches_runs_and_stops(node_type: NodeType, rpc_port: u16) -> Result<()> {
    let workspace = Workspace::new()?;
    let node = workspace.add_node(&format!("{node_type} lifecycle"), node_type, rpc_port)?;
    let mut supervisor = ProcessSupervisor::default();

    let (outcome, inputs) = launch(&workspace, &mut supervisor, &node, LaunchAction::Start)?;
    let NodeLaunchOutcome::Started {
        pid,
        log_path,
        replaced_unmanaged,
    } = outcome
    else {
        bail!("{node_type} did not start: {outcome:?}");
    };
    assert_eq!(log_path, inputs.log_path);
    assert!(!replaced_unmanaged, "a first start has nothing to replace");

    // The managed config was rendered before the launch, in the generator's
    // format, where the launch command tells the client to read it.
    let format = ConfigFormat::for_node_type(node_type);
    assert_eq!(
        inputs
            .config_path
            .extension()
            .and_then(|extension| extension.to_str()),
        Some(format.extension()),
        "{node_type} managed config path {}",
        inputs.config_path.display()
    );
    parses_as(format, &std::fs::read_to_string(&inputs.config_path)?)?;
    if node_type == NodeType::NeoCli {
        // neo-cli reads config.json from its working directory.
        assert_eq!(
            inputs.config_path,
            inputs.plan.working_dir.join("config.json")
        );
    } else {
        let config_argument = inputs.config_path.display().to_string();
        assert!(
            inputs.plan.args.contains(&config_argument),
            "{node_type}'s launch command does not name its managed config: {:?}",
            inputs.plan.args
        );
    }

    // The workspace says Running, with the pid the operating system is running.
    let running = workspace.stored(&node.id)?;
    assert_eq!(running.status, NodeStatus::Running);
    assert_eq!(running.pid, Some(pid));
    assert!(
        process_is_live(pid),
        "{node_type} is recorded as running as pid {pid}, which is not alive"
    );
    assert!(supervisor.is_managing(&node.id));

    let stopped = stop(&workspace, &mut supervisor, &node.id)?;
    assert_eq!(stopped.pid, pid);
    // Unix asks the runtime to exit before it would kill it; Windows has no
    // such request, so the only stop there is a forced one.
    assert_eq!(
        (stopped.graceful, stopped.forced),
        (cfg!(unix), !cfg!(unix))
    );

    assert!(
        !process_is_live(pid),
        "pid {pid} outlived the stop that reported it stopped"
    );
    assert!(!supervisor.is_managing(&node.id));
    let settled = workspace.stored(&node.id)?;
    assert_eq!(settled.status, NodeStatus::Stopped);
    assert_eq!(settled.pid, None, "a stopped node keeps no pid");

    // Both halves are on the record in the node's log.
    let log = std::fs::read_to_string(&inputs.log_path)?;
    assert_eq!(log.matches("== NeoNexus launch").count(), 1, "{log}");
    assert!(log.contains(&format!("pid: {pid}")), "{log}");
    assert_eq!(log.matches("== NeoNexus stop").count(), 1, "{log}");
    Ok(())
}

#[test]
fn neo_cli_node_launches_runs_and_stops() -> Result<()> {
    launches_runs_and_stops(NodeType::NeoCli, 31_332)
}

#[test]
fn neo_go_node_launches_runs_and_stops() -> Result<()> {
    launches_runs_and_stops(NodeType::NeoGo, 31_432)
}

#[test]
fn neo_rs_node_launches_runs_and_stops() -> Result<()> {
    launches_runs_and_stops(NodeType::NeoRs, 31_532)
}

#[test]
fn neox_geth_node_launches_runs_and_stops() -> Result<()> {
    launches_runs_and_stops(NodeType::NeoXGeth, 31_632)
}

#[test]
fn neox_rs_node_launches_runs_and_stops() -> Result<()> {
    launches_runs_and_stops(NodeType::NeoXReth, 31_732)
}

#[test]
fn restart_replaces_the_running_process_and_records_the_new_pid() -> Result<()> {
    let workspace = Workspace::new()?;
    let node = workspace.add_node("restart", NodeType::NeoGo, 32_332)?;
    let mut supervisor = ProcessSupervisor::default();

    let first = started_pid(launch(&workspace, &mut supervisor, &node, LaunchAction::Start)?.0)?;
    let running = workspace.stored(&node.id)?;
    let (outcome, inputs) = launch(&workspace, &mut supervisor, &running, LaunchAction::Restart)?;
    let second = started_pid(outcome)?;

    assert_ne!(first, second, "restart kept the old process");
    assert!(
        !process_is_live(first),
        "restart left the old process {first} running"
    );
    assert!(process_is_live(second));
    let restarted = workspace.stored(&node.id)?;
    assert_eq!(restarted.status, NodeStatus::Running);
    assert_eq!(restarted.pid, Some(second));
    let log = std::fs::read_to_string(&inputs.log_path)?;
    assert_eq!(log.matches("== NeoNexus launch").count(), 2, "{log}");

    stop(&workspace, &mut supervisor, &node.id)?;
    assert!(!process_is_live(second));
    Ok(())
}

/// One supervisor, every node type at once, plus a node whose runtime is not
/// installed: the broken node fails on its own, and stopping one node leaves
/// the others running.
#[test]
fn every_node_type_runs_side_by_side_and_a_broken_runtime_fails_alone() -> Result<()> {
    let workspace = Workspace::new()?;
    let mut supervisor = ProcessSupervisor::default();

    let mut running = Vec::new();
    for (index, node_type) in NodeType::ALL.into_iter().enumerate() {
        let rpc_port = 33_000 + 10 * u16::try_from(index)?;
        let node = workspace.add_node(&format!("fleet {node_type}"), node_type, rpc_port)?;
        let pid = started_pid(launch(&workspace, &mut supervisor, &node, LaunchAction::Start)?.0)?;
        running.push((node.id, pid));
    }

    let missing = workspace.path().join("not-installed").join("neo-node");
    let broken = workspace.add_node_with_runtime("broken", NodeType::NeoRs, 33_100, missing)?;
    let (outcome, _) = launch(&workspace, &mut supervisor, &broken, LaunchAction::Start)?;
    assert!(
        matches!(outcome, NodeLaunchOutcome::Failed { .. }),
        "a runtime that does not exist was reported started: {outcome:?}"
    );
    assert_eq!(workspace.stored(&broken.id)?.status, NodeStatus::Error);

    let pids: BTreeSet<u32> = running.iter().map(|(_, pid)| *pid).collect();
    assert_eq!(
        pids.len(),
        NodeType::ALL.len(),
        "each node has its own process"
    );
    let managed: BTreeSet<String> = supervisor.managed_node_ids().into_iter().collect();
    let expected: BTreeSet<String> = running.iter().map(|(id, _)| id.clone()).collect();
    assert_eq!(managed, expected);
    for (id, pid) in &running {
        assert!(
            process_is_live(*pid),
            "pid {pid} died beside the broken node"
        );
        let stored = workspace.stored(id)?;
        assert_eq!(stored.status, NodeStatus::Running);
        assert_eq!(stored.pid, Some(*pid));
    }

    let Some(((first_id, first_pid), others)) = running.split_first() else {
        bail!("no node was started");
    };
    stop(&workspace, &mut supervisor, first_id)?;
    assert!(!process_is_live(*first_pid));
    for (id, pid) in others {
        assert!(
            process_is_live(*pid),
            "stopping one node also ended pid {pid}"
        );
        assert_eq!(workspace.stored(id)?.status, NodeStatus::Running);
    }

    for (id, pid) in others {
        stop(&workspace, &mut supervisor, id)?;
        assert!(!process_is_live(*pid));
    }
    Ok(())
}

#[test]
fn a_missing_runtime_fails_in_error_and_the_node_starts_once_rebound() -> Result<()> {
    let workspace = Workspace::new()?;
    let missing = workspace.path().join("not-installed").join("neo-node");
    let node =
        workspace.add_node_with_runtime("uninstalled", NodeType::NeoRs, 34_332, missing.clone())?;
    let mut supervisor = ProcessSupervisor::default();

    let (outcome, _) = launch(&workspace, &mut supervisor, &node, LaunchAction::Start)?;
    let NodeLaunchOutcome::Failed { message } = outcome else {
        bail!("a runtime that does not exist was reported started: {outcome:?}");
    };
    assert!(
        message.contains("failed to start") && message.contains(&missing.display().to_string()),
        "the failure must name the runtime it could not start: {message}"
    );
    let failed = workspace.stored(&node.id)?;
    assert_eq!(failed.status, NodeStatus::Error);
    assert_eq!(failed.pid, None);
    assert!(!supervisor.is_managing(&node.id));

    // The failed attempt released its launch claim: once a runtime that exists
    // is bound, the same node starts.
    workspace
        .repository()
        .rebind_node_runtime(&node.id, stub_runtime_binary(), Vec::new())?;
    let rebound = workspace.stored(&node.id)?;
    let pid = started_pid(launch(&workspace, &mut supervisor, &rebound, LaunchAction::Start)?.0)?;
    assert!(process_is_live(pid));
    assert_eq!(workspace.stored(&node.id)?.pid, Some(pid));

    stop(&workspace, &mut supervisor, &node.id)?;
    Ok(())
}

/// A supervisor that goes away takes the processes it still manages with it,
/// so a shell that exits leaves no orphaned node runtimes behind.
#[test]
fn dropping_the_supervisor_ends_every_process_it_still_manages() -> Result<()> {
    let workspace = Workspace::new()?;
    let mut supervisor = ProcessSupervisor::default();
    let mut pids = Vec::new();
    for (name, node_type, rpc_port) in [
        ("drop one", NodeType::NeoCli, 35_332),
        ("drop two", NodeType::NeoXGeth, 35_432),
    ] {
        let node = workspace.add_node(name, node_type, rpc_port)?;
        pids.push(started_pid(
            launch(&workspace, &mut supervisor, &node, LaunchAction::Start)?.0,
        )?);
    }
    assert!(pids.iter().all(|&pid| process_is_live(pid)));

    drop(supervisor);

    for pid in pids {
        assert!(
            !process_is_live(pid),
            "pid {pid} outlived the supervisor that managed it"
        );
    }
    Ok(())
}
