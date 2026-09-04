use neo_nexus::{
    launch::LaunchPlan,
    node_lifecycle::{execute_node_launch, LaunchAction, NodeLaunchOutcome},
    repository::Repository,
    supervisor::{process_is_live, ProcessSupervisor},
    types::{Network, NewNode, NodeConfig, NodeStatus, NodeType, StorageEngine},
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::Duration,
};

fn fixture_binary() -> &'static Path {
    static FIXTURE: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    &FIXTURE
        .get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let source = dir.path().join("fixture.rs");
            let binary = dir.path().join(if cfg!(windows) {
                "fixture.exe"
            } else {
                "fixture"
            });
            std::fs::write(
                &source,
                "fn main() { loop { std::thread::sleep(std::time::Duration::from_millis(50)); } }",
            )
            .unwrap();
            let mut command = Command::new("rustc");
            command
                .arg(&source)
                .args(["--edition", "2021", "-o"])
                .arg(&binary);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x0800_0000);
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            (dir, binary)
        })
        .1
}

fn workspace() -> (tempfile::TempDir, Repository, NodeConfig, LaunchPlan) {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("node.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "persistence-rejected".into(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: fixture_binary().into(),
            args: vec![],
            runtime_version: "fixture".into(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 13332,
            p2p_port: 13333,
            ws_port: None,
        })
        .unwrap();
    let plan = LaunchPlan {
        binary_path: node.binary_path.clone(),
        args: vec![],
        working_dir: dir.path().to_path_buf(),
        managed_config_path: None,
        display_command: "owned fixture".into(),
    };
    (dir, repository, node, plan)
}

#[test]
fn a_rejected_running_status_stops_the_real_new_child() -> anyhow::Result<()> {
    let (dir, repository, node, plan) = workspace();
    let connection = rusqlite::Connection::open(repository.db_path()).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER reject_running BEFORE UPDATE OF status ON nodes
        WHEN NEW.status='running' BEGIN SELECT RAISE(ABORT,'status unavailable'); END;",
        )
        .unwrap();
    let mut supervisor = ProcessSupervisor::with_stop_grace_period(Duration::from_millis(100));
    let outcome = execute_node_launch(
        &repository,
        &mut supervisor,
        &node,
        &plan,
        dir.path().join("fixture.log"),
        LaunchAction::Start,
        None,
    );
    let NodeLaunchOutcome::Failed { message } = outcome else {
        anyhow::bail!("database rejection must fail the launch")
    };
    let pid: u32 = message
        .strip_prefix("failed to persist running process ")
        .unwrap()
        .split(':')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(message.contains("was stopped"), "{message}");
    assert!(
        !process_is_live(pid),
        "new child {pid} must not become an unrecorded orphan"
    );
    assert!(!supervisor.is_managing(&node.id));
    assert_eq!(
        repository.list_nodes().unwrap()[0].status,
        NodeStatus::Error
    );
    assert!(repository.list_nodes().unwrap()[0].pid.is_none());
    Ok(())
}

#[test]
fn a_rejected_starting_reservation_never_spawns_a_child() {
    let (dir, repository, node, plan) = workspace();
    let connection = rusqlite::Connection::open(repository.db_path()).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER reject_starting BEFORE UPDATE OF status ON nodes
        WHEN NEW.status='starting' BEGIN SELECT RAISE(ABORT,'reservation unavailable'); END;",
        )
        .unwrap();
    let log = dir.path().join("never-started.log");
    let mut supervisor = ProcessSupervisor::default();
    let result = execute_node_launch(
        &repository,
        &mut supervisor,
        &node,
        &plan,
        &log,
        LaunchAction::Start,
        None,
    );
    assert!(
        matches!(result, NodeLaunchOutcome::Failed { message } if message.contains("could not reserve node launch"))
    );
    assert!(!log.exists());
    assert!(!supervisor.is_managing(&node.id));
    assert_eq!(
        repository.list_nodes().unwrap()[0].status,
        NodeStatus::Stopped
    );
}

#[test]
fn stale_configuration_cannot_launch_after_a_concurrent_replacement() {
    let (dir, repository, node, plan) = workspace();
    let connection = rusqlite::Connection::open(repository.db_path()).unwrap();
    connection
        .execute(
            "UPDATE nodes SET runtime_version='replacement' WHERE id=?1",
            [&node.id],
        )
        .unwrap();
    let log = dir.path().join("never-started.log");
    let mut supervisor = ProcessSupervisor::default();
    let result = execute_node_launch(
        &repository,
        &mut supervisor,
        &node,
        &plan,
        &log,
        LaunchAction::Start,
        None,
    );
    assert!(
        matches!(result, NodeLaunchOutcome::Failed { message } if message.contains("configuration or lifecycle changed"))
    );
    assert!(!log.exists());
    assert!(!supervisor.is_managing(&node.id));
    assert_eq!(
        repository.list_nodes().unwrap()[0].runtime_version,
        "replacement"
    );
    assert_eq!(
        repository.list_nodes().unwrap()[0].status,
        NodeStatus::Stopped
    );
}
