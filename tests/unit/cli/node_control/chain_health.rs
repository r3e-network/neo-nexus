//! Asking what the chain says, without a browser.
//!
//! The exit code carries the answer, so these tests check it as carefully as
//! the text: a cron job or a deployment gate acts on the code, and a command
//! that always exits 0 is a check nobody can build on.

use super::super::*;

use crate::observe::{Evidence, HealthState, NextStep, NodeHealth, NodeSample, Observation};

const NOW: u64 = 1_770_000_000;

fn workspace() -> Result<(tempfile::TempDir, std::path::PathBuf, Repository)> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("neonexus.db");
    let repository = Repository::open(&path)?;
    Ok((dir, path, repository))
}

fn node(repository: &Repository, name: &str, rpc_port: u16) -> Result<String> {
    Ok(repository
        .create_node(NewNode {
            name: name.to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Private,
            binary_path: "/opt/neo/neo-go".into(),
            args: Vec::new(),
            runtime_version: "0.122".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port,
            p2p_port: rpc_port + 1,
            ws_port: None,
        })?
        .id)
}

fn record_sample(repository: &Repository, node_id: &str, height: u64) -> Result<()> {
    let mut sample = NodeSample::not_observable(node_id, NOW);
    sample.endpoint = "http://127.0.0.1:30332".to_string();
    sample.head_ok = true;
    sample.head_latency_ms = Some(14);
    sample.block_height = Observation::Known(
        height,
        Evidence::recorded("getblockcount", "result", height.to_string(), "", NOW),
    );
    repository.record_node_sample(&sample)?;
    Ok(())
}

fn record_health(repository: &Repository, node_id: &str, state: HealthState) -> Result<()> {
    repository.save_node_health(&NodeHealth {
        node_id: node_id.to_string(),
        state,
        since_unix: NOW,
        evaluated_at_unix: NOW,
        reason: format!("{} for a reason worth reading", state.label()),
        scope: None,
        cause: None,
        next: NextStep::here("Read the log", format!("/logs?node={node_id}")),
    })?;
    Ok(())
}

/// The exit code is the answer. A gate that has to grep the text of a command
/// that always succeeds is not a gate.
#[test]
fn fleet_health_exits_non_zero_exactly_when_something_needs_attention() -> Result<()> {
    let (_dir, path, repository) = workspace()?;
    let healthy = node(&repository, "healthy", 30332)?;
    record_sample(&repository, &healthy, 8_421)?;
    record_health(&repository, &healthy, HealthState::Healthy)?;
    drop(repository);

    let db = path.display().to_string();
    let CliAction::PrintWithExitCode { exit_code, text } =
        action_from_args(["neo-nexus", "--fleet-health", &db])?
    else {
        unreachable!("--fleet-health prints with an exit code")
    };
    assert_eq!(exit_code, 0, "{text}");
    assert!(text.contains("healthy"), "{text}");
    assert!(text.contains("none needing attention"), "{text}");

    let repository = Repository::open(&path)?;
    let stalled = node(&repository, "stalled", 30432)?;
    record_sample(&repository, &stalled, 12)?;
    record_health(&repository, &stalled, HealthState::Stalled)?;
    drop(repository);

    let CliAction::PrintWithExitCode { exit_code, text } =
        action_from_args(["neo-nexus", "--fleet-health", &db])?
    else {
        unreachable!("--fleet-health prints with an exit code")
    };
    assert_eq!(exit_code, 1, "{text}");
    assert!(text.contains("1 of 2 nodes need attention"), "{text}");
    // Worst first, so reading from the top answers "what do I look at".
    let stalled_line = text.find("stalled").unwrap_or(usize::MAX);
    let healthy_line = text.find("healthy ").unwrap_or(0);
    assert!(stalled_line < healthy_line, "{text}");
    Ok(())
}

/// A node nobody has judged is not a node that is well, and the table must not
/// let the two share a blank.
#[test]
fn a_node_with_no_verdict_says_so_rather_than_printing_a_blank() -> Result<()> {
    let (_dir, path, repository) = workspace()?;
    node(&repository, "fresh", 30332)?;
    drop(repository);

    let db = path.display().to_string();
    let CliAction::PrintWithExitCode { exit_code, text } =
        action_from_args(["neo-nexus", "--fleet-health", &db])?
    else {
        unreachable!("--fleet-health prints with an exit code")
    };
    assert_eq!(exit_code, 0, "not having looked is not a failure");
    assert!(text.contains("not judged"), "{text}");
    assert!(text.contains("not checked"), "{text}");
    assert!(text.contains("1 have no verdict yet"), "{text}");
    Ok(())
}

/// A single node's report carries the verdict, the readings it was drawn from
/// and the history, because those are the three things an operator reaches for
/// in that order.
#[test]
fn node_health_reports_the_verdict_its_readings_and_its_history() -> Result<()> {
    let (_dir, path, repository) = workspace()?;
    let id = node(&repository, "rpc-1", 30332)?;
    record_sample(&repository, &id, 8_421)?;
    record_health(&repository, &id, HealthState::Stalled)?;
    repository.record_health_transition(&crate::observe::HealthTransition {
        node_id: id.clone(),
        at_unix: NOW,
        from: Some(HealthState::Healthy),
        to: HealthState::Stalled,
        reason: "height 8421 has not advanced in 600s".to_string(),
    })?;
    drop(repository);

    let db = path.display().to_string();
    let CliAction::PrintWithExitCode { exit_code, text } =
        action_from_args(["neo-nexus", "--node-health", &db, "rpc-1"])?
    else {
        unreachable!("--node-health prints with an exit code")
    };
    assert_eq!(exit_code, 1, "a stalled node is a failing check");
    assert!(text.contains("health: Stalled"), "{text}");
    assert!(text.contains("block-height: 8421"), "{text}");
    assert!(text.contains("rpc-round-trip: 14 ms"), "{text}");
    assert!(text.contains("next: Read the log"), "{text}");
    assert!(text.contains("Healthy → Stalled"), "{text}");
    Ok(())
}

/// A reading that was not taken must not arrive as a zero in machine-readable
/// output either. A consumer that reads a null peer count as zero peers has
/// invented an incident.
#[test]
fn unread_measurements_are_null_in_json_never_zero() -> Result<()> {
    let (_dir, path, repository) = workspace()?;
    let id = node(&repository, "rpc-1", 30332)?;
    record_sample(&repository, &id, 8_421)?;
    record_health(&repository, &id, HealthState::Healthy)?;
    drop(repository);

    let db = path.display().to_string();
    let CliAction::PrintWithExitCode { exit_code, text } =
        action_from_args(["neo-nexus", "--node-health-json", &db, "rpc-1"])?
    else {
        unreachable!("--node-health-json prints with an exit code")
    };
    assert_eq!(exit_code, 0);
    let payload: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(payload["block_height"], 8_421);
    assert!(
        payload["peers_connected"].is_null(),
        "an unread peer count must not be 0: {text}"
    );
    assert!(payload["observed_magic"].is_null(), "{text}");
    assert_eq!(payload["health"]["state"], "healthy");
    assert_eq!(payload["health"]["needs_attention"], false);
    Ok(())
}

/// The JSON form carries the same verdict as the table, so a dashboard built on
/// one cannot disagree with an operator reading the other.
#[test]
fn the_json_fleet_report_agrees_with_the_table() -> Result<()> {
    let (_dir, path, repository) = workspace()?;
    let id = node(&repository, "isolated", 30332)?;
    record_sample(&repository, &id, 5)?;
    record_health(&repository, &id, HealthState::Isolated)?;
    drop(repository);

    let db = path.display().to_string();
    let CliAction::PrintWithExitCode { exit_code, text } =
        action_from_args(["neo-nexus", "--fleet-health-json", &db])?
    else {
        unreachable!("--fleet-health-json prints with an exit code")
    };
    assert_eq!(exit_code, 1);
    let payload: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(payload["needing_attention"], 1);
    assert_eq!(payload["nodes"][0]["name"], "isolated");
    assert_eq!(payload["nodes"][0]["health"]["state"], "isolated");
    Ok(())
}

/// Naming a node that does not exist is an error, not an empty report that
/// reads like a clean bill of health.
#[test]
fn asking_about_a_node_that_does_not_exist_fails_loudly() -> Result<()> {
    let (_dir, path, _repository) = workspace()?;
    let db = path.display().to_string();
    assert!(action_from_args(["neo-nexus", "--node-health", &db, "ghost"]).is_err());
    Ok(())
}
