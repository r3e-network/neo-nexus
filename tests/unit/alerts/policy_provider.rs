use super::*;

#[test]
fn alert_policy_matches_threshold_and_enabled_state() {
    let mut policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("https://hooks.example.com/neo".to_string()),
        timeout_seconds: 5,
    };

    assert!(!should_route_alert(&policy, &event(EventSeverity::Info)));
    assert!(should_route_alert(&policy, &event(EventSeverity::Warning)));
    assert!(should_route_alert(&policy, &event(EventSeverity::Critical)));

    policy.enabled = false;
    assert!(!should_route_alert(
        &policy,
        &event(EventSeverity::Critical)
    ));
}

#[test]
fn alert_provider_round_trips_from_storage_labels() -> anyhow::Result<()> {
    assert_eq!(AlertProvider::from_str("generic")?, AlertProvider::Generic);
    assert_eq!(AlertProvider::from_str("slack")?, AlertProvider::Slack);
    assert_eq!(AlertProvider::from_str("discord")?, AlertProvider::Discord);
    assert_eq!(
        AlertProvider::from_str("telegram")?,
        AlertProvider::Telegram
    );
    assert_eq!(
        AlertProvider::from_str("pagerduty")?,
        AlertProvider::PagerDuty
    );
    assert_eq!(
        AlertProvider::from_str("opsgenie")?,
        AlertProvider::Opsgenie
    );
    assert_eq!(
        AlertProvider::from_str("datadog-events")?,
        AlertProvider::Datadog
    );
    assert!(AlertProvider::ALL.contains(&AlertProvider::PagerDuty));
    assert!(AlertProvider::ALL.contains(&AlertProvider::Opsgenie));
    assert!(AlertProvider::ALL.contains(&AlertProvider::Datadog));
    assert_eq!(AlertProvider::Slack.to_string(), "slack");
    assert_eq!(AlertProvider::PagerDuty.to_string(), "pagerduty");
    assert_eq!(AlertProvider::Opsgenie.to_string(), "opsgenie");
    assert_eq!(AlertProvider::Datadog.to_string(), "datadog");

    Ok(())
}

#[test]
fn alert_policy_validation_requires_target_when_enabled() {
    let policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Critical,
        webhook_url: None,
        timeout_seconds: 5,
    };
    assert!(policy.validation_message().is_some());

    let disabled = AlertRoutingPolicy::default();
    assert!(disabled.validation_message().is_none());

    let telegram_without_chat = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Telegram,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("https://api.telegram.org/bot123:abc/sendMessage".to_string()),
        timeout_seconds: 5,
    };
    assert!(telegram_without_chat
        .validation_message()
        .is_some_and(|message| message.contains("chat_id")));

    let pagerduty_without_key = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::PagerDuty,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("https://events.pagerduty.com/v2/enqueue".to_string()),
        timeout_seconds: 5,
    };
    assert!(pagerduty_without_key
        .validation_message()
        .is_some_and(|message| message.contains("routing_key")));

    let opsgenie_without_key = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Opsgenie,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("https://api.opsgenie.com/v2/alerts".to_string()),
        timeout_seconds: 5,
    };
    assert!(opsgenie_without_key
        .validation_message()
        .is_some_and(|message| message.contains("api_key")));

    let datadog_without_key = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Datadog,
        min_severity: EventSeverity::Warning,
        webhook_url: Some(
            "https://event-management-intake.datadoghq.com/api/v2/events".to_string(),
        ),
        timeout_seconds: 5,
    };
    assert!(datadog_without_key
        .validation_message()
        .is_some_and(|message| message.contains("api_key")));
}

#[test]
fn new_runtime_event_still_compiles_for_alert_context() {
    let event = NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind: EventKind::WatchdogExhausted,
        severity: EventSeverity::Critical,
        message: "restart budget exhausted".to_string(),
    };
    assert_eq!(event.severity, EventSeverity::Critical);
}
