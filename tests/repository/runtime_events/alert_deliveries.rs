use super::*;

#[test]
fn records_and_prunes_alert_delivery_history() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let event = repository
        .record_event_at(
            NewRuntimeEvent {
                node_id: Some("node-alert".to_string()),
                node_name: Some("alerted".to_string()),
                kind: EventKind::RpcHealthChecked,
                severity: EventSeverity::Critical,
                message: "rpc unreachable".to_string(),
            },
            100,
        )
        .unwrap();

    let failed = AlertDeliveryReport {
        event_id: event.id,
        route_label: "webhook".to_string(),
        target: "https://hooks.example.com".to_string(),
        status: AlertDeliveryStatus::Failed,
        http_status: Some(500),
        message: "server rejected alert".to_string(),
    };
    let delivered = AlertDeliveryReport {
        event_id: event.id,
        route_label: "webhook".to_string(),
        target: "https://hooks.example.com".to_string(),
        status: AlertDeliveryStatus::Delivered,
        http_status: Some(204),
        message: "accepted".to_string(),
    };

    repository.record_alert_delivery(&failed).unwrap();
    repository.record_alert_delivery(&delivered).unwrap();

    let history = repository.list_alert_deliveries(10).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].status, AlertDeliveryStatus::Delivered);
    assert_eq!(history[0].http_status, Some(204));
    assert_eq!(history[1].status, AlertDeliveryStatus::Failed);

    let deleted = repository.prune_alert_deliveries_keep_recent(1).unwrap();
    assert_eq!(deleted, 1);
    let pruned = repository.list_alert_deliveries(10).unwrap();
    assert_eq!(pruned.len(), 1);
    assert_eq!(pruned[0].status, AlertDeliveryStatus::Delivered);
}
