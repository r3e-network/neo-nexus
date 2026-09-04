#![cfg(windows)]

use std::{
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

use neo_nexus::{
    launch::LaunchPlan,
    repository::Repository,
    supervisor::{process_is_live, ManagedProcessSpec, ProcessSupervisor},
    types::{Network, NewNode, NodeConfig, NodeStatus, NodeType, StorageEngine},
};

// A tiny real console node: CTRL_BREAK sets a flag, and the main loop flushes
// an observable shutdown marker before returning. Stubborn mode acknowledges
// the event without exiting. No installed Go/.NET runtime is needed.
fn fixture_binary() -> &'static Path {
    static FIXTURE: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    &FIXTURE
        .get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let source = dir.path().join("console_node.rs");
            let binary = dir.path().join("console_node.exe");
            std::fs::write(
                &source,
                r#"
use std::{sync::atomic::{AtomicBool, Ordering}, time::Duration};
static STOP: AtomicBool = AtomicBool::new(false);
extern "system" fn handler(event: u32) -> i32 {
    if event != 1 { return 0; }
    STOP.store(true, Ordering::SeqCst); 1
}
#[link(name = "kernel32")]
extern "system" {
    fn SetConsoleCtrlHandler(handler: extern "system" fn(u32) -> i32, add: i32) -> i32;
    fn GetConsoleProcessList(pids: *mut u32, length: u32) -> u32;
    fn AllocConsole() -> i32;
    fn GetConsoleWindow() -> *mut std::ffi::c_void;
}
#[link(name = "user32")]
extern "system" { fn ShowWindow(window: *mut std::ffi::c_void, command: i32) -> i32; }
fn main() {
    let stubborn = std::env::args().any(|arg| arg == "--stubborn");
    let mut pids = [0; 4];
    unsafe {
        if GetConsoleProcessList(pids.as_mut_ptr(), 4) == 0 {
            assert_ne!(AllocConsole(), 0);
            ShowWindow(GetConsoleWindow(), 0);
        }
        assert_ne!(SetConsoleCtrlHandler(handler, 1), 0);
    }
    std::fs::write("ready", "ready").unwrap();
    if let Ok(value) = std::env::var("NEONEXUS_TEST_CHILD_HOME") {
        std::fs::write("child-home", value).unwrap();
    }
    if std::env::args().any(|arg| arg == "--exit-early") { std::process::exit(7); }
    while stubborn || !STOP.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(10));
    }
    std::fs::write("graceful-stop", "flushed").unwrap();
}
"#,
            )
            .unwrap();
            let result = Command::new("rustc")
                .arg(&source)
                .args(["--edition", "2021", "-o"])
                .arg(&binary)
                .creation_flags(0x0800_0000)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            (dir, binary)
        })
        .1
}

fn node(repo: &Repository, binary: &Path, stubborn: bool) -> NodeConfig {
    repo.create_node(NewNode {
        name: "console-node".into(),
        node_type: NodeType::NeoGo,
        network: Network::Testnet,
        binary_path: binary.into(),
        args: if stubborn {
            vec!["--stubborn".into()]
        } else {
            vec![]
        },
        runtime_version: "fixture".into(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port: 34332,
        p2p_port: 34333,
        ws_port: None,
    })
    .unwrap()
}

fn cli(db: &Path, operation: &str) -> Output {
    let db_path = db.to_path_buf();
    let operation_owned = operation.to_string();
    let (sender, receiver) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let output = Command::new(env!("CARGO_BIN_EXE_neo-nexus"))
            .arg(operation_owned)
            .arg(db_path)
            .arg("console-node")
            // A controller without the target console exercises the helper.
            .creation_flags(0x0800_0000)
            .output();
        let _ = sender.send(output);
    });
    receiver
        .recv_timeout(Duration::from_secs(20))
        .unwrap_or_else(|_| panic!("{operation} did not close its output pipes within 20 seconds"))
        .unwrap()
}

struct StopNodeOnDrop<'a>(&'a Repository);

impl Drop for StopNodeOnDrop<'_> {
    fn drop(&mut self) {
        if let Ok(nodes) = self.0.list_nodes() {
            let mut system = sysinfo::System::new_all();
            system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            for node in nodes {
                if let Some(process) = node
                    .pid
                    .and_then(|pid| system.process(sysinfo::Pid::from_u32(pid)))
                {
                    if process.exe().is_some_and(|path| path == fixture_binary()) {
                        process.kill();
                    }
                }
            }
        }
    }
}

fn wait_ready(work: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !work.join("ready").exists() {
        assert!(Instant::now() < deadline, "node did not become ready");
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn recorded_windows_node_flushes_on_break_from_another_cli_console() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("workspace.db");
    let repo = Repository::open(&db).unwrap();
    let node = node(&repo, fixture_binary(), false);
    let _cleanup = StopNodeOnDrop(&repo);
    let started = cli(&db, "--node-start");
    assert!(
        started.status.success(),
        "{}{}",
        String::from_utf8_lossy(&started.stdout),
        String::from_utf8_lossy(&started.stderr)
    );
    let work = dir.path().join("nodes").join(&node.id);
    wait_ready(&work);
    let pid = repo.list_nodes().unwrap()[0].pid.unwrap();
    let stopped = cli(&db, "--node-stop");
    assert!(
        stopped.status.success(),
        "{}{}",
        String::from_utf8_lossy(&stopped.stdout),
        String::from_utf8_lossy(&stopped.stderr)
    );
    assert!(
        work.join("graceful-stop").exists(),
        "CTRL_BREAK must let the node flush"
    );
    assert!(!process_is_live(pid));
    assert_eq!(repo.list_nodes().unwrap()[0].status, NodeStatus::Stopped);
}

#[test]
fn cli_restart_replaces_the_previous_recorded_windows_process() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("workspace.db");
    let repo = Repository::open(&db).unwrap();
    let node = node(&repo, fixture_binary(), false);
    let _cleanup = StopNodeOnDrop(&repo);
    assert!(cli(&db, "--node-start").status.success());
    let work = dir.path().join("nodes").join(&node.id);
    wait_ready(&work);
    let first = repo.list_nodes().unwrap()[0].pid.unwrap();
    std::fs::remove_file(work.join("ready")).unwrap();
    let restarted = cli(&db, "--node-restart");
    assert!(
        restarted.status.success(),
        "{}{}",
        String::from_utf8_lossy(&restarted.stdout),
        String::from_utf8_lossy(&restarted.stderr)
    );
    let second = repo.list_nodes().unwrap()[0].pid.unwrap();
    wait_ready(&work);
    assert_ne!(first, second);
    assert!(
        !process_is_live(first),
        "CLI restart must not leave a duplicate node"
    );
    assert!(process_is_live(second));
    assert!(cli(&db, "--node-stop").status.success());
}

#[test]
fn windows_stop_force_kills_a_stubborn_managed_node() {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(dir.path().join("workspace.db")).unwrap();
    let node = node(&repo, fixture_binary(), true);
    let plan = LaunchPlan {
        binary_path: node.binary_path.clone(),
        args: node.args.clone(),
        working_dir: dir.path().join("work"),
        managed_config_path: None,
        display_command: "console fixture".into(),
    };
    let mut supervisor = ProcessSupervisor::with_stop_grace_period(Duration::from_millis(100));
    let start = supervisor
        .start(&node, &plan, dir.path().join("node.log"))
        .unwrap();
    wait_ready(&plan.working_dir);
    let stop = supervisor.stop(&node.id).unwrap().unwrap();
    assert!(stop.forced);
    assert!(!stop.graceful);
    assert!(!process_is_live(start.pid));
}

#[test]
fn internal_break_rejects_zero_without_broadcasting() {
    let output = Command::new(env!("CARGO_BIN_EXE_neo-nexus"))
        .args(["--internal-console-break", "0"])
        .creation_flags(0x0800_0000)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn child_environment_does_not_change_the_parent_or_leak_into_logs() {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(dir.path().join("workspace.db")).unwrap();
    let node = node(&repo, fixture_binary(), true);
    let plan = LaunchPlan {
        binary_path: node.binary_path.clone(),
        args: node.args.clone(),
        working_dir: dir.path().join("work"),
        managed_config_path: None,
        display_command: "console fixture".into(),
    };
    let before = std::env::var_os("NEONEXUS_TEST_CHILD_HOME");
    let log = dir.path().join("node.log");
    let mut supervisor = ProcessSupervisor::with_stop_grace_period(Duration::ZERO);
    supervisor
        .start_process_with_env(
            &ManagedProcessSpec::for_node(&node, &plan),
            &log,
            &[(
                "NEONEXUS_TEST_CHILD_HOME".into(),
                "isolated-child-secret".into(),
            )],
        )
        .unwrap();
    wait_ready(&plan.working_dir);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !plan.working_dir.join("child-home").exists() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        std::fs::read_to_string(plan.working_dir.join("child-home")).unwrap(),
        "isolated-child-secret"
    );
    assert_eq!(std::env::var_os("NEONEXUS_TEST_CHILD_HOME"), before);
    supervisor.stop(&node.id).unwrap();
    assert!(!std::fs::read_to_string(log)
        .unwrap()
        .contains("isolated-child-secret"));
}

#[test]
fn stopping_an_already_exited_child_preserves_its_actual_exit_code() {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(dir.path().join("workspace.db")).unwrap();
    let node = node(&repo, fixture_binary(), false);
    let plan = LaunchPlan {
        binary_path: node.binary_path.clone(),
        args: vec!["--exit-early".into()],
        working_dir: dir.path().join("work"),
        managed_config_path: None,
        display_command: "console fixture".into(),
    };
    let mut supervisor = ProcessSupervisor::default();
    let start = supervisor
        .start(&node, &plan, dir.path().join("node.log"))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while process_is_live(start.pid) {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    let stopped = supervisor.stop(&node.id).unwrap().unwrap();
    assert_eq!(stopped.exit_code, Some(7));
    assert!(!stopped.forced);
    assert!(!stopped.graceful);
}
