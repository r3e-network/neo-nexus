use super::*;

#[cfg(unix)]
#[test]
fn supervisor_restarts_after_child_exits() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "short lived".to_string(),
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
        args: vec!["-c".to_string(), "echo restartable".to_string()],
        working_dir: temp_dir.path().join("work"),
        managed_config_path: None,
        display_command: "/bin/sh -c 'echo restartable'".to_string(),
    };
    let log_path = temp_dir.path().join("logs").join("restartable.log");
    let mut supervisor = ProcessSupervisor::default();

    supervisor.start(&node, &plan, &log_path).unwrap();
    wait_for_reaped_exit(&mut supervisor, &node.id);
    supervisor.start(&node, &plan, &log_path).unwrap();
    wait_for_reaped_exit(&mut supervisor, &node.id);
    let _ = supervisor.stop(&node.id).unwrap();

    let text = std::fs::read_to_string(&log_path).unwrap();
    assert_eq!(text.matches("== NeoNexus launch").count(), 2);
    assert_eq!(text.matches("restartable").count(), 4);
}

#[cfg(unix)]
#[test]
fn supervisor_restart_replaces_running_child_process() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "long lived".to_string(),
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
            "echo booted; while true; do sleep 1; done".to_string(),
        ],
        working_dir: temp_dir.path().join("work"),
        managed_config_path: None,
        display_command: "/bin/sh -c 'echo booted; while true; do sleep 1; done'".to_string(),
    };
    let log_path = temp_dir.path().join("logs").join("restart.log");
    let mut supervisor = ProcessSupervisor::default();

    let first = supervisor.start(&node, &plan, &log_path).unwrap();
    thread::sleep(Duration::from_millis(100));
    let second = supervisor.restart(&node, &plan, &log_path).unwrap();
    thread::sleep(Duration::from_millis(100));
    let _ = supervisor.stop(&node.id).unwrap();

    assert_ne!(first.pid, second.pid);
    let text = std::fs::read_to_string(&log_path).unwrap();
    assert_eq!(text.matches("== NeoNexus launch").count(), 2);
    assert_eq!(text.matches("== NeoNexus stop").count(), 2);
    assert!(text.contains(&format!("pid: {}", first.pid)));
    assert!(text.contains(&format!("pid: {}", second.pid)));
}

#[cfg(unix)]
fn wait_for_reaped_exit(supervisor: &mut ProcessSupervisor, node_id: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let exits = supervisor.reap_finished().unwrap();
        if exits.iter().any(|exit| exit.process_id == node_id) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {node_id} to exit"
        );
        thread::sleep(Duration::from_millis(10));
    }
}
