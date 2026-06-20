use crate::*;

#[test]
fn launch_planner_attaches_managed_config_for_neo_rs() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let config_path = PathBuf::from("/tmp/neo-rs.toml");

    let plan = LaunchPlanner::plan(&node, &config_path, "/tmp/neo-rs-work");

    assert_eq!(plan.args, ["--config", "/tmp/neo-rs.toml"]);
    assert_eq!(plan.working_dir, PathBuf::from("/tmp/neo-rs-work"));
    assert_eq!(plan.managed_config_path, Some(config_path));
    assert!(plan.display_command.contains("--config /tmp/neo-rs.toml"));
}

#[test]
fn launch_planner_publishes_neo_cli_config_json_in_workdir() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let config_path = PathBuf::from("/tmp/neo-cli-work/config.json");

    let plan = LaunchPlanner::plan(&node, &config_path, "/tmp/neo-cli-work");

    assert!(plan.args.is_empty());
    assert_eq!(plan.working_dir, PathBuf::from("/tmp/neo-cli-work"));
    assert_eq!(plan.managed_config_path, Some(config_path));
}

#[test]
fn launch_planner_preserves_existing_config_arg() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = repo
        .create_node(NewNode {
            name: "neo-rs custom".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("/usr/local/bin/neo-node"),
            args: vec!["--config".to_string(), "custom.toml".to_string()],
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 20332,
            p2p_port: 20333,
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

    let plan = LaunchPlanner::plan(&node, "/tmp/generated.toml", "/tmp/neo-rs-custom-work");

    assert_eq!(plan.args, ["--config", "custom.toml"]);
    assert_eq!(plan.working_dir, PathBuf::from("/tmp/neo-rs-custom-work"));
    assert_eq!(plan.managed_config_path, None);
}

#[test]
fn launch_config_arg_detection_matches_supported_runtimes() {
    assert!(runtime_args_include_config(
        NodeType::NeoRs,
        &["--config=custom.toml".to_string()]
    ));
    assert!(runtime_args_include_config(
        NodeType::NeoRs,
        &["-c".to_string(), "custom.toml".to_string()]
    ));
    assert!(runtime_args_include_config(
        NodeType::NeoGo,
        &[
            "node".to_string(),
            "--config-file".to_string(),
            "custom.yml".to_string()
        ]
    ));
    assert!(runtime_args_include_config(
        NodeType::NeoGo,
        &["node".to_string(), "--config-path=custom.yml".to_string()]
    ));
    assert!(!runtime_args_include_config(
        NodeType::NeoCli,
        &["--config".to_string(), "custom.json".to_string()]
    ));
}

#[test]
fn launch_planner_injects_managed_config_for_neo_go() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-go", NodeType::NeoGo);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let plan = LaunchPlanner::plan(&node, "/tmp/generated.yml", "/tmp/neo-go-work");

    assert_eq!(plan.args, ["node", "--config-file", "/tmp/generated.yml"]);
    assert_eq!(plan.working_dir, PathBuf::from("/tmp/neo-go-work"));
    assert_eq!(
        plan.managed_config_path,
        Some(PathBuf::from("/tmp/generated.yml"))
    );
}

#[test]
fn launch_planner_preserves_existing_neo_go_config_arg() {
    let repo = create_repo();
    let node_id = repo
        .create_node(NewNode {
            name: "neo-go custom".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/usr/local/bin/neo-go"),
            args: vec![
                "node".to_string(),
                "--config-file".to_string(),
                "custom.yml".to_string(),
            ],
            runtime_version: "v0.110.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
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

    let plan = LaunchPlanner::plan(&node, "/tmp/generated.yml", "/tmp/neo-go-work");

    assert_eq!(plan.args, ["node", "--config-file", "custom.yml"]);
    assert_eq!(plan.working_dir, PathBuf::from("/tmp/neo-go-work"));
    assert_eq!(plan.managed_config_path, None);
}
