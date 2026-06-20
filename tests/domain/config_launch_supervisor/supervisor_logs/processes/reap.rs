use super::*;

#[cfg(unix)]
#[test]
fn supervisor_reaps_finished_processes_with_exit_codes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "failed child".to_string(),
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
        args: vec!["-c".to_string(), "exit 7".to_string()],
        working_dir: temp_dir.path().join("work"),
        managed_config_path: None,
        display_command: "/bin/sh -c 'exit 7'".to_string(),
    };
    let log_path = temp_dir.path().join("logs").join("exited.log");
    let mut supervisor = ProcessSupervisor::default();

    let start = supervisor.start(&node, &plan, &log_path).unwrap();
    thread::sleep(Duration::from_millis(100));
    let exits = supervisor.reap_finished().unwrap();

    assert_eq!(exits.len(), 1);
    assert_eq!(exits[0].node_id, node.id);
    assert_eq!(exits[0].pid, start.pid);
    assert_eq!(exits[0].exit_code, Some(7));
    assert!(supervisor.reap_finished().unwrap().is_empty());
}
