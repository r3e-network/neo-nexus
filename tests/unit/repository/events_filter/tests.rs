use std::path::PathBuf;

use crate::{
    events::{EventKind, EventSeverity, NewRuntimeEvent, RuntimeEventFilter},
    repository::Repository,
    types::{Network, NewNode, NodeType, StorageEngine},
};

fn workspace() -> (tempfile::TempDir, Repository) {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("n.db")).unwrap();
    (dir, repository)
}

fn node(repository: &Repository, name: &str, rpc_port: u16) -> (String, String) {
    let node = repository
        .create_node(NewNode {
            name: name.to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Private,
            binary_path: PathBuf::from("/opt/neo/neo-go"),
            args: Vec::new(),
            runtime_version: "0.122".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port,
            p2p_port: rpc_port + 1,
            ws_port: None,
        })
        .unwrap();
    (node.id, node.name)
}

fn record(repository: &Repository, node: Option<(&str, &str)>, kind: EventKind, message: &str) {
    repository
        .record_event(NewRuntimeEvent {
            node_id: node.map(|(id, _)| id.to_string()),
            node_name: node.map(|(_, name)| name.to_string()),
            kind,
            severity: EventSeverity::Info,
            message: message.to_string(),
        })
        .unwrap();
}

/// The journal holds 93 kinds and the page offered no way to pick one, so
/// "show me every watchdog decision" meant guessing a substring that happened
/// to appear in the message text.
#[test]
fn the_journal_can_be_narrowed_to_one_kind() {
    let (_dir, repository) = workspace();
    let (id, name) = node(&repository, "rpc-1", 30332);
    record(
        &repository,
        Some((&id, &name)),
        EventKind::NodeStarted,
        "up",
    );
    record(
        &repository,
        Some((&id, &name)),
        EventKind::WatchdogExhausted,
        "gave up",
    );
    record(
        &repository,
        Some((&id, &name)),
        EventKind::NodeStopped,
        "down",
    );

    let filter = RuntimeEventFilter::of_kind(EventKind::WatchdogExhausted, 50);
    assert_eq!(repository.count_events(&filter).unwrap(), 1);
    let found = repository.list_events(filter).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, EventKind::WatchdogExhausted);
}

/// Every other per-node surface accepts a node scope; the journal did not, so
/// the node page's "view the journal" link dumped the whole workspace and left
/// the operator to find their node in it.
#[test]
fn the_journal_can_be_narrowed_to_one_node() {
    let (_dir, repository) = workspace();
    let (mine, my_name) = node(&repository, "mine", 30332);
    let (theirs, their_name) = node(&repository, "theirs", 30432);
    record(
        &repository,
        Some((&mine, &my_name)),
        EventKind::NodeStarted,
        "up",
    );
    record(
        &repository,
        Some((&theirs, &their_name)),
        EventKind::NodeStarted,
        "up",
    );
    record(
        &repository,
        None,
        EventKind::BackupExported,
        "workspace-wide",
    );

    let filter = RuntimeEventFilter::new(None, "", 50).for_node(&mine);
    let found = repository.list_events(filter).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].node_id.as_deref(), Some(mine.as_str()));
}

/// An empty node scope is not a node. Treating one as a filter would return
/// nothing and read to an operator as "this node has no history".
#[test]
fn an_empty_node_scope_matches_everything_rather_than_nothing() {
    let (_dir, repository) = workspace();
    let (id, name) = node(&repository, "rpc-1", 30332);
    record(
        &repository,
        Some((&id, &name)),
        EventKind::NodeStarted,
        "up",
    );
    record(
        &repository,
        None,
        EventKind::BackupExported,
        "workspace-wide",
    );

    let filter = RuntimeEventFilter::new(None, "", 50).for_node("   ");
    assert_eq!(repository.list_events(filter).unwrap().len(), 2);
}

/// The scopes compose, and the count agrees with the rows — a page that shows
/// "3 matching" above one row has lost the operator's trust in both numbers.
#[test]
fn the_scopes_compose_and_the_count_matches_the_rows() {
    let (_dir, repository) = workspace();
    let (mine, my_name) = node(&repository, "mine", 30332);
    let (theirs, their_name) = node(&repository, "theirs", 30432);
    record(
        &repository,
        Some((&mine, &my_name)),
        EventKind::NodeStarted,
        "up",
    );
    record(
        &repository,
        Some((&mine, &my_name)),
        EventKind::NodeStopped,
        "down",
    );
    record(
        &repository,
        Some((&theirs, &their_name)),
        EventKind::NodeStarted,
        "up",
    );

    let filter = RuntimeEventFilter::of_kind(EventKind::NodeStarted, 50).for_node(&mine);
    assert_eq!(repository.count_events(&filter).unwrap(), 1);
    assert_eq!(repository.list_events(filter).unwrap().len(), 1);
}
