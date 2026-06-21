use super::*;

#[test]
fn alert_routing_policy_draft_validates_and_normalizes_target() {
    let policy = AlertRoutingPolicy {
        enabled: false,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Warning,
        webhook_url: None,
        timeout_seconds: 5,
    };
    let draft = AlertRoutingPolicyDraft::from_policy(&policy);

    assert!(draft.validation_message().is_none());
    assert!(!draft.differs_from(&policy));
    assert_eq!(draft.to_policy(), policy);

    let missing_target = AlertRoutingPolicyDraft {
        enabled: true,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Critical,
        webhook_url: String::new(),
        timeout_seconds: 5,
    };
    assert!(missing_target.validation_message().is_some());

    let secure_target = AlertRoutingPolicyDraft {
        enabled: true,
        provider: AlertProvider::Telegram,
        min_severity: EventSeverity::Critical,
        webhook_url: " https://api.telegram.org/bot123:abc/sendMessage?chat_id=-100123 "
            .to_string(),
        timeout_seconds: 99,
    };
    let normalized = secure_target.to_policy();
    assert_eq!(
        normalized.webhook_url.as_deref(),
        Some("https://api.telegram.org/bot123:abc/sendMessage?chat_id=-100123")
    );
    assert_eq!(
        normalized.timeout_seconds,
        AlertRoutingPolicy::MAX_TIMEOUT_SECONDS
    );

    assert_telegram_target_requires_chat_id();
    assert_provider_targets_normalize();
    assert_datadog_target_requires_api_key();
}

fn assert_telegram_target_requires_chat_id() {
    let missing_chat = AlertRoutingPolicyDraft {
        enabled: true,
        provider: AlertProvider::Telegram,
        min_severity: EventSeverity::Critical,
        webhook_url: "https://api.telegram.org/bot123:abc/sendMessage".to_string(),
        timeout_seconds: 5,
    };
    assert!(missing_chat
        .validation_message()
        .is_some_and(|message| message.contains("chat_id")));
}

fn assert_provider_targets_normalize() {
    let pagerduty = AlertRoutingPolicyDraft {
        enabled: true,
        provider: AlertProvider::PagerDuty,
        min_severity: EventSeverity::Critical,
        webhook_url: " https://events.pagerduty.com/v2/enqueue?routing_key=abc123 ".to_string(),
        timeout_seconds: 5,
    };
    assert_eq!(
        pagerduty.to_policy().webhook_url.as_deref(),
        Some("https://events.pagerduty.com/v2/enqueue?routing_key=abc123")
    );

    let opsgenie = AlertRoutingPolicyDraft {
        enabled: true,
        provider: AlertProvider::Opsgenie,
        min_severity: EventSeverity::Critical,
        webhook_url: " https://api.opsgenie.com/v2/alerts?api_key=abc123 ".to_string(),
        timeout_seconds: 5,
    };
    assert_eq!(
        opsgenie.to_policy().webhook_url.as_deref(),
        Some("https://api.opsgenie.com/v2/alerts?api_key=abc123")
    );

    let datadog = AlertRoutingPolicyDraft {
        enabled: true,
        provider: AlertProvider::Datadog,
        min_severity: EventSeverity::Critical,
        webhook_url: " https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123 "
            .to_string(),
        timeout_seconds: 5,
    };
    assert_eq!(
        datadog.to_policy().webhook_url.as_deref(),
        Some("https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123")
    );
}

fn assert_datadog_target_requires_api_key() {
    let datadog_without_key = AlertRoutingPolicyDraft {
        enabled: true,
        provider: AlertProvider::Datadog,
        min_severity: EventSeverity::Critical,
        webhook_url: "https://event-management-intake.datadoghq.com/api/v2/events".to_string(),
        timeout_seconds: 5,
    };
    assert!(datadog_without_key
        .validation_message()
        .is_some_and(|message| message.contains("api_key")));
}
