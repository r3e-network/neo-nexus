use super::*;

#[test]
fn loads_and_persists_alert_routing_policy_without_backing_up_webhook_secret() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    assert_eq!(
        repository.load_alert_routing_policy().unwrap(),
        AlertRoutingPolicy::default()
    );

    let custom = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Telegram,
        min_severity: EventSeverity::Critical,
        webhook_url: Some(
            "https://api.telegram.org/bot123:abc/sendMessage?chat_id=-100123".to_string(),
        ),
        timeout_seconds: 9,
    };
    repository.save_alert_routing_policy(custom).unwrap();
    let loaded = repository.load_alert_routing_policy().unwrap();

    assert!(loaded.enabled);
    assert_eq!(loaded.provider, AlertProvider::Telegram);
    assert_eq!(loaded.min_severity, EventSeverity::Critical);
    assert_eq!(
        loaded.webhook_url.as_deref(),
        Some("https://api.telegram.org/bot123:abc/sendMessage?chat_id=-100123")
    );
    assert_eq!(loaded.timeout_seconds, 9);

    let settings = repository.list_workspace_settings_for_backup().unwrap();
    assert!(settings
        .iter()
        .all(|setting| !setting.key.starts_with("alert_routing.")));

    let insecure = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Slack,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("http://hooks.example.com/neo".to_string()),
        timeout_seconds: 5,
    };
    assert!(repository.save_alert_routing_policy(insecure).is_err());

    let telegram_without_chat = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Telegram,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("https://api.telegram.org/bot123:abc/sendMessage".to_string()),
        timeout_seconds: 5,
    };
    assert!(repository
        .save_alert_routing_policy(telegram_without_chat)
        .is_err());

    let pagerduty = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::PagerDuty,
        min_severity: EventSeverity::Critical,
        webhook_url: Some("https://events.pagerduty.com/v2/enqueue?routing_key=abc123".to_string()),
        timeout_seconds: 7,
    };
    repository.save_alert_routing_policy(pagerduty).unwrap();
    let loaded = repository.load_alert_routing_policy().unwrap();
    assert_eq!(loaded.provider, AlertProvider::PagerDuty);
    assert_eq!(
        loaded.webhook_url.as_deref(),
        Some("https://events.pagerduty.com/v2/enqueue?routing_key=abc123")
    );

    let opsgenie = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Opsgenie,
        min_severity: EventSeverity::Critical,
        webhook_url: Some("https://api.opsgenie.com/v2/alerts?api_key=abc123".to_string()),
        timeout_seconds: 8,
    };
    repository.save_alert_routing_policy(opsgenie).unwrap();
    let loaded = repository.load_alert_routing_policy().unwrap();
    assert_eq!(loaded.provider, AlertProvider::Opsgenie);
    assert_eq!(
        loaded.webhook_url.as_deref(),
        Some("https://api.opsgenie.com/v2/alerts?api_key=abc123")
    );

    let datadog = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Datadog,
        min_severity: EventSeverity::Warning,
        webhook_url: Some(
            "https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123".to_string(),
        ),
        timeout_seconds: 6,
    };
    repository.save_alert_routing_policy(datadog).unwrap();
    let loaded = repository.load_alert_routing_policy().unwrap();
    assert_eq!(loaded.provider, AlertProvider::Datadog);
    assert_eq!(
        loaded.webhook_url.as_deref(),
        Some("https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123")
    );
}
