use super::super::*;
use super::neo_rs_app_node;

#[test]
fn missing_process_reconciliation_marks_stale_running_nodes_stopped() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    let node =
        app.repository
            .create_node(neo_rs_app_node("missing process", 30332, 30333, None))?;
    let missing_pid = u32::MAX - 7;
    app.repository
        .update_node_status(&node.id, NodeStatus::Running, Some(missing_pid))?;
    app.reload_nodes();
    app.refresh_metrics_now();

    assert_eq!(app.metrics_snapshot.missing_processes.len(), 1);

    app.reconcile_missing_process_records();

    let reconciled = app
        .repository
        .list_nodes()?
        .into_iter()
        .find(|candidate| candidate.id == node.id)
        .ok_or_else(|| anyhow::anyhow!("node should still exist"))?;
    assert_eq!(reconciled.status, NodeStatus::Stopped);
    assert_eq!(reconciled.pid, None);
    assert!(app.metrics_snapshot.missing_processes.is_empty());
    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| { notice.contains("Runtime state reconciled: 1 missing process") }));
    let events = app.repository.list_events(RuntimeEventFilter::new(
        None,
        "runtime-state-reconciled",
        10,
    ))?;
    assert!(events.iter().any(|event| {
        event.kind == EventKind::RuntimeStateReconciled
            && event.node_id.as_deref() == Some(node.id.as_str())
    }));

    Ok(())
}

#[cfg(unix)]
#[test]
fn restart_selected_node_replaces_running_process_and_audits() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let node = repository.create_node(NewNode {
        name: "restartable node".to_string(),
        node_type: NodeType::NeoCli,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: vec![
            "-c".to_string(),
            "echo app-restart-test; while true; do sleep 1; done".to_string(),
        ],
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 20332,
        p2p_port: 20333,
        ws_port: None,
    })?;
    let mut app = NeoNexusApp::new(repository);
    app.selected_node = Some(node.id.clone());

    app.start_selected_node();
    let first_pid = app
        .selected_node()
        .and_then(|node| node.pid)
        .ok_or_else(|| anyhow::anyhow!("start should record a PID"))?;

    app.restart_selected_node();
    let restarted = app
        .selected_node()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("node should remain selected"))?;
    let second_pid = restarted
        .pid
        .ok_or_else(|| anyhow::anyhow!("restart should record a PID"))?;

    assert_eq!(restarted.status, NodeStatus::Running);
    assert_ne!(first_pid, second_pid);
    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("restarted with PID")));
    let events = app
        .repository
        .list_events(RuntimeEventFilter::new(None, "node-restarted", 10))?;
    assert!(events.iter().any(|event| {
        event.kind == EventKind::NodeRestarted && event.node_id.as_deref() == Some(node.id.as_str())
    }));

    app.stop_selected_node();

    Ok(())
}
