use super::*;

#[cfg(unix)]
#[test]
fn supervisor_writes_child_output_to_log_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "local logger".to_string(),
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
            "echo stdout-line; echo stderr-line >&2".to_string(),
        ],
        working_dir: temp_dir.path().join("work"),
        managed_config_path: None,
        display_command: "/bin/sh -c 'echo stdout-line; echo stderr-line >&2'".to_string(),
    };
    let log_path = temp_dir.path().join("logs").join("node.log");
    let mut supervisor = ProcessSupervisor::default();

    let ProcessStart {
        pid: _,
        log_path: started_log_path,
    } = supervisor.start(&node, &plan, &log_path).unwrap();
    thread::sleep(Duration::from_millis(100));
    let _ = supervisor.stop(&node.id).unwrap();

    let text = std::fs::read_to_string(&log_path).unwrap();
    assert_eq!(started_log_path, log_path);
    assert!(text.contains("local logger"));
    assert!(text.contains("pid: "));
    assert!(text.contains("working-dir: "));
    assert!(text.contains("stdout-line"));
    assert!(text.contains("stderr-line"));
    assert!(text.contains("== NeoNexus stop"));
    assert!(text.contains("stop-mode: "));
}
