use super::*;

#[test]
fn records_runtime_events_in_recent_order_and_keeps_deleted_node_snapshot() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "audited".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: "/usr/local/bin/neo-go".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();

    repository
        .record_event_at(
            NewRuntimeEvent {
                node_id: Some(node.id.clone()),
                node_name: Some(node.name.clone()),
                kind: EventKind::NodeStarted,
                severity: EventSeverity::Info,
                message: "started".to_string(),
            },
            100,
        )
        .unwrap();
    repository
        .record_event_at(
            NewRuntimeEvent {
                node_id: Some(node.id.clone()),
                node_name: Some(node.name.clone()),
                kind: EventKind::WatchdogExhausted,
                severity: EventSeverity::Critical,
                message: "exhausted".to_string(),
            },
            101,
        )
        .unwrap();
    repository.delete_node(&node.id).unwrap();

    let events = repository.list_recent_events(10).unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, EventKind::WatchdogExhausted);
    assert_eq!(events[0].severity, EventSeverity::Critical);
    assert_eq!(events[0].node_id.as_deref(), Some(node.id.as_str()));
    assert_eq!(events[0].node_name.as_deref(), Some("audited"));
    assert_eq!(events[1].kind, EventKind::NodeStarted);
}

#[test]
fn filters_and_prunes_runtime_events() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    for index in 0..6 {
        repository
            .record_event_at(
                NewRuntimeEvent {
                    node_id: Some(format!("node-{index}")),
                    node_name: Some(if index % 2 == 0 {
                        "alpha".to_string()
                    } else {
                        "beta".to_string()
                    }),
                    kind: if index % 2 == 0 {
                        EventKind::NodeStarted
                    } else {
                        EventKind::WatchdogScheduled
                    },
                    severity: if index % 2 == 0 {
                        EventSeverity::Info
                    } else {
                        EventSeverity::Warning
                    },
                    message: if index == 5 {
                        "peer timeout".to_string()
                    } else {
                        format!("event {index}")
                    },
                },
                100 + index,
            )
            .unwrap();
    }

    let warning_timeout = RuntimeEventFilter::new(Some(EventSeverity::Warning), "timeout", 10);
    let filtered = repository.list_events(warning_timeout.clone()).unwrap();
    assert_eq!(repository.count_events(&warning_timeout).unwrap(), 1);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].message, "peer timeout");

    let removed = repository.prune_events_keep_recent(2).unwrap();
    let remaining = repository.list_recent_events(10).unwrap();

    assert_eq!(removed, 4);
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining[0].occurred_at_unix, 105);
    assert_eq!(remaining[1].occurred_at_unix, 104);
}
