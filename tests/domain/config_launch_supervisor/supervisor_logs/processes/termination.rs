use super::*;

#[cfg(unix)]
#[test]
fn supervisor_stops_child_gracefully_before_force_kill() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "graceful child".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/bin/sh"),
            args: Vec::new(),
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap()
        .id;
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let plan = LaunchPlan {
        binary_path: PathBuf::from("/bin/sh"),
        args: vec![
            "-c".to_string(),
            "trap 'echo graceful-stop; exit 0' TERM; echo ready; while :; do :; done".to_string(),
        ],
        working_dir: temp_dir.path().join("work"),
        managed_config_path: None,
        display_command: "/bin/sh -c 'trap ... TERM; echo ready; while :; do :; done'".to_string(),
    };
    let log_path = temp_dir.path().join("logs").join("graceful.log");
    let mut supervisor = ProcessSupervisor::with_stop_grace_period(Duration::from_millis(250));

    supervisor.start(&node, &plan, &log_path).unwrap();
    thread::sleep(Duration::from_millis(50));
    let stop = supervisor.stop(&node.id).unwrap().unwrap();

    assert!(stop.graceful);
    assert!(!stop.forced);
    assert_eq!(stop.exit_code, Some(0));
    let text = std::fs::read_to_string(&log_path).unwrap();
    assert!(text.contains("graceful-stop"));
    assert!(text.contains("stop-mode: graceful"));
}

#[cfg(unix)]
#[test]
fn supervisor_force_kills_child_after_grace_period() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "stubborn child".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/bin/sh"),
            args: Vec::new(),
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap()
        .id;
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let plan = LaunchPlan {
        binary_path: PathBuf::from("/bin/sh"),
        args: vec![
            "-c".to_string(),
            "trap '' TERM; echo stubborn-ready; while :; do :; done".to_string(),
        ],
        working_dir: temp_dir.path().join("work"),
        managed_config_path: None,
        display_command: "/bin/sh -c 'trap ignore TERM; while :; do :; done'".to_string(),
    };
    let log_path = temp_dir.path().join("logs").join("forced.log");
    let mut supervisor = ProcessSupervisor::with_stop_grace_period(Duration::from_millis(50));

    supervisor.start(&node, &plan, &log_path).unwrap();
    thread::sleep(Duration::from_millis(50));
    let stop = supervisor.stop(&node.id).unwrap().unwrap();

    assert!(stop.forced);
    assert!(!stop.graceful);
    assert_eq!(stop.exit_code, None);
    let text = std::fs::read_to_string(&log_path).unwrap();
    assert!(text.contains("stubborn-ready"));
    assert!(text.contains("stop-mode: forced"));
}
