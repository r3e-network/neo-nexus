use super::*;

#[test]
fn alert_delivery_history_filters_by_status() {
    let deliveries = [
        delivery(1, AlertDeliveryStatus::Delivered, "datadog", None, "ok"),
        delivery(2, AlertDeliveryStatus::Failed, "slack", None, "failed"),
        delivery(
            3,
            AlertDeliveryStatus::Skipped,
            "pagerduty",
            None,
            "skipped",
        ),
        delivery(4, AlertDeliveryStatus::Failed, "opsgenie", None, "failed"),
    ];

    let all = filter_alert_deliveries(&deliveries, &AlertDeliveryFilter::new(None, ""));
    assert_eq!(all.len(), 4);

    let failed = filter_alert_deliveries(
        &deliveries,
        &AlertDeliveryFilter::new(Some(AlertDeliveryStatus::Failed), ""),
    );
    assert_eq!(
        failed
            .iter()
            .map(|delivery| delivery.id)
            .collect::<Vec<_>>(),
        vec![2, 4]
    );

    let skipped = filter_alert_deliveries(
        &deliveries,
        &AlertDeliveryFilter::new(Some(AlertDeliveryStatus::Skipped), ""),
    );
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].id, 3);
}

#[test]
fn alert_delivery_history_filters_by_query_text() {
    let deliveries = [
        delivery(
            1,
            AlertDeliveryStatus::Delivered,
            "Datadog",
            Some(202),
            "ok",
        ),
        delivery(
            2,
            AlertDeliveryStatus::Failed,
            "Slack",
            Some(503),
            "critical rpc failure",
        ),
        delivery(3, AlertDeliveryStatus::Skipped, "PagerDuty", None, "muted"),
    ];

    assert_ids(&deliveries, AlertDeliveryFilter::new(None, "datadog"), &[1]);
    assert_ids(
        &deliveries,
        AlertDeliveryFilter::new(None, "HTTP 503"),
        &[2],
    );
    assert_ids(&deliveries, AlertDeliveryFilter::new(None, "pager"), &[3]);
    assert_ids(
        &deliveries,
        AlertDeliveryFilter::new(None, "critical"),
        &[2],
    );
    assert_ids(&deliveries, AlertDeliveryFilter::new(None, "skipped"), &[3]);
    assert_ids(&deliveries, AlertDeliveryFilter::new(None, "30"), &[3]);
}

#[test]
fn alert_delivery_history_combines_status_and_query() {
    let deliveries = [
        delivery(1, AlertDeliveryStatus::Delivered, "Slack", Some(202), "ok"),
        delivery(2, AlertDeliveryStatus::Failed, "Slack", Some(500), "failed"),
        delivery(
            3,
            AlertDeliveryStatus::Failed,
            "Telegram",
            Some(500),
            "failed",
        ),
    ];
    let filter = AlertDeliveryFilter::new(Some(AlertDeliveryStatus::Failed), "slack");

    assert_ids(&deliveries, filter, &[2]);
}

fn assert_ids(deliveries: &[AlertDelivery], filter: AlertDeliveryFilter, ids: &[i64]) {
    let actual = filter_alert_deliveries(deliveries, &filter)
        .iter()
        .map(|delivery| delivery.id)
        .collect::<Vec<_>>();
    assert_eq!(actual.as_slice(), ids);
}

fn delivery(
    id: i64,
    status: AlertDeliveryStatus,
    route_label: &str,
    http_status: Option<u16>,
    message: &str,
) -> AlertDelivery {
    AlertDelivery {
        id,
        event_id: id * 10,
        attempted_at_unix: 1_800_000_000 + id as u64,
        route_label: route_label.to_string(),
        target: format!("https://alerts.example/{route_label}/{id}"),
        status,
        http_status,
        message: message.to_string(),
    }
}
