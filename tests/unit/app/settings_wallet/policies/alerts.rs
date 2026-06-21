use super::*;

#[test]
fn alert_routing_policy_action_persists_and_audits() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.alert_routing_policy_draft = AlertRoutingPolicyDraft {
        enabled: true,
        provider: AlertProvider::Telegram,
        min_severity: EventSeverity::Critical,
        webhook_url: "http://127.0.0.1:9/bot123:abc/sendMessage?chat_id=-100123".to_string(),
        timeout_seconds: 3,
    };
    app.save_alert_routing_policy();

    assert!(app.alert_routing_policy.enabled);
    assert_eq!(
        app.alert_routing_policy.min_severity,
        EventSeverity::Critical
    );
    assert_eq!(app.alert_routing_policy.provider, AlertProvider::Telegram);
    assert_eq!(
        app.repository
            .load_alert_routing_policy()?
            .webhook_url
            .as_deref(),
        Some("http://127.0.0.1:9/bot123:abc/sendMessage?chat_id=-100123")
    );
    let events = app
        .repository
        .list_events(RuntimeEventFilter::new(None, "alert-routing", 10))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::AlertRoutingPolicyUpdated));

    Ok(())
}

#[test]
fn alert_routing_policy_preview_uses_draft_without_sending() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.alert_routing_policy_draft = AlertRoutingPolicyDraft {
        enabled: false,
        provider: AlertProvider::Datadog,
        min_severity: EventSeverity::Critical,
        webhook_url: "https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123"
            .to_string(),
        timeout_seconds: 5,
    };
    app.preview_alert_routing_policy_draft();

    let preview = app
        .last_alert_preview
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("expected alert preview report"))?;
    assert_eq!(preview.status, "ready");
    assert_eq!(preview.provider, "datadog");
    assert_eq!(preview.severity, "critical");
    assert_eq!(
        preview.endpoint,
        "https://event-management-intake.datadoghq.com"
    );
    assert_eq!(preview.headers[0].name, "DD-API-KEY");
    assert_eq!(preview.headers[0].value, "<redacted>");
    assert!(!preview.payload_json.contains("dd123"));
    assert!(app.alert_preview_matches_draft());
    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("Alert preview ready")));

    app.alert_routing_policy_draft.min_severity = EventSeverity::Warning;
    assert!(app.last_alert_preview.is_some());
    assert!(!app.alert_preview_matches_draft());

    app.alert_routing_policy_draft.webhook_url =
        "https://event-management-intake.datadoghq.com/api/v2/events".to_string();
    app.preview_alert_routing_policy_draft();

    assert!(app.last_alert_preview.is_none());
    assert!(app.last_alert_preview_policy.is_none());
    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("api_key")));

    Ok(())
}
