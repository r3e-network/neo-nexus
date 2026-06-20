use super::*;

#[test]
fn webhook_urls_are_normalized_and_restricted() -> anyhow::Result<()> {
    assert_eq!(
        normalized_webhook_url(" https://hooks.example.com/path/ ")?,
        "https://hooks.example.com/path"
    );
    assert!(normalized_webhook_url("http://127.0.0.1:9090/alert").is_ok());
    assert!(normalized_webhook_url("http://hooks.example.com/alert").is_err());
    assert!(normalized_webhook_url("https://user:pass@hooks.example.com/alert").is_err());
    assert!(normalized_webhook_url("https://hooks.example.com/alert#secret").is_err());

    Ok(())
}

#[test]
fn telegram_payload_escapes_html_and_target_requires_chat_id() {
    let mut event = event(EventSeverity::Warning);
    event.node_name = Some("validator <1>".to_string());
    event.message = "RPC <down> & retry \"now\"".to_string();

    let payload = telegram_alert_payload(&event, "2.5.3", "-100123");
    let text = payload["text"].as_str().unwrap_or_default();

    assert!(text.contains("validator &lt;1&gt;"));
    assert!(text.contains("RPC &lt;down&gt; &amp; retry &quot;now&quot;"));
    assert!(
        telegram_target("https://api.telegram.org/bot123:abc/sendMessage?chat_id=-100123").is_ok()
    );
    assert!(
        telegram_target("https://api.telegram.org/bot123:abc/getUpdates?chat_id=-100123").is_err()
    );
    assert!(telegram_target("https://api.telegram.org/bot123:abc/sendMessage").is_err());
}

#[test]
fn pagerduty_target_requires_events_api_v2_enqueue_and_routing_key() {
    assert!(pagerduty_target("https://events.pagerduty.com/v2/enqueue?routing_key=abc123").is_ok());
    assert!(pagerduty_target("https://events.pagerduty.com/v2/enqueue").is_err());
    assert!(
        pagerduty_target("https://events.pagerduty.com/v2/change/enqueue?routing_key=abc123")
            .is_err()
    );
    assert!(pagerduty_target("https://example.com/v2/enqueue?routing_key=abc123").is_err());
}

#[test]
fn opsgenie_target_requires_alerts_api_v2_and_builds_genie_key_header() -> anyhow::Result<()> {
    let target = opsgenie_target("https://api.opsgenie.com/v2/alerts?api_key=abc123")?;
    assert_eq!(target.endpoint_url, "https://api.opsgenie.com/v2/alerts");
    assert_eq!(target.api_key, "abc123");
    assert!(opsgenie_target("https://api.opsgenie.com/v2/alerts").is_err());
    assert!(opsgenie_target("https://api.opsgenie.com/v2/heartbeats?api_key=abc123").is_err());
    assert!(opsgenie_target("https://example.com/v2/alerts?api_key=abc123").is_err());

    let policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Opsgenie,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("https://api.opsgenie.com/v2/alerts?api_key=abc123".to_string()),
        timeout_seconds: 5,
    };
    let request = alert_delivery_request(
        policy.provider,
        &event(EventSeverity::Critical),
        "2.5.3",
        policy.webhook_url.as_deref().unwrap_or_default(),
    )?;

    assert_eq!(request.endpoint_url, "https://api.opsgenie.com/v2/alerts");
    assert!(request
        .headers
        .iter()
        .any(|(name, value)| name == "Authorization" && value == "GenieKey abc123"));
    assert_eq!(request.payload["priority"], "P1");

    Ok(())
}

#[test]
fn datadog_target_requires_events_api_v2_and_builds_api_key_header() -> anyhow::Result<()> {
    let target = datadog_target(
        "https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123",
    )?;
    assert_eq!(
        target.endpoint_url,
        "https://event-management-intake.datadoghq.com/api/v2/events"
    );
    assert_eq!(target.api_key, "dd123");
    assert!(datadog_target("https://event-management-intake.datadoghq.com/api/v2/events").is_err());
    assert!(datadog_target(
        "https://event-management-intake.datadoghq.com/api/v1/events?api_key=dd123"
    )
    .is_err());
    assert!(datadog_target("https://api.datadoghq.com/api/v2/events?api_key=dd123").is_err());

    let policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Datadog,
        min_severity: EventSeverity::Warning,
        webhook_url: Some(
            "https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123".to_string(),
        ),
        timeout_seconds: 5,
    };
    let request = alert_delivery_request(
        policy.provider,
        &event(EventSeverity::Critical),
        "2.5.3",
        policy.webhook_url.as_deref().unwrap_or_default(),
    )?;

    assert_eq!(
        request.endpoint_url,
        "https://event-management-intake.datadoghq.com/api/v2/events"
    );
    assert!(request
        .headers
        .iter()
        .any(|(name, value)| name == "DD-API-KEY" && value == "dd123"));
    assert_eq!(request.payload["data"]["type"], "event");
    assert_eq!(
        request.payload["data"]["attributes"]["attributes"]["status"],
        "error"
    );

    Ok(())
}
