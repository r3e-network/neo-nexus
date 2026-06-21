use super::*;

#[test]
fn alert_preview_json_validates_datadog_without_sending() -> Result<()> {
    let action = action_from_args([
        "neo-nexus",
        "--alert-preview-json",
        "datadog",
        "https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123",
        "critical",
        "RPC",
        "health",
        "unreachable",
    ])?;
    let CliAction::Print(text) = action else {
        anyhow::bail!("expected JSON alert preview");
    };
    let value: serde_json::Value = serde_json::from_str(&text)?;
    let report = &value["report"];
    let payload_json = report["payload_json"].as_str().unwrap_or_default();

    assert_eq!(value["status"], "ready");
    assert_eq!(value["success"], true);
    assert_eq!(report["status"], "ready");
    assert_eq!(report["provider"], "datadog");
    assert_eq!(
        report["target"],
        "https://event-management-intake.datadoghq.com"
    );
    assert_eq!(
        report["endpoint"],
        "https://event-management-intake.datadoghq.com"
    );
    assert_eq!(report["headers"][0]["name"], "DD-API-KEY");
    assert_eq!(report["headers"][0]["value"], "<redacted>");
    assert!(payload_json.contains("\"category\": \"alert\""));
    assert!(payload_json.contains("RPC health unreachable"));
    assert!(!text.contains("dd123"));
    Ok(())
}

#[test]
fn alert_preview_text_redacts_pagerduty_routing_key() -> Result<()> {
    let action = action_from_args([
        "neo-nexus",
        "--alert-preview",
        "pagerduty",
        "https://events.pagerduty.com/v2/enqueue?routing_key=pager-secret",
        "critical",
        "validator",
        "offline",
    ])?;
    let CliAction::Print(text) = action else {
        anyhow::bail!("expected text alert preview");
    };

    assert!(text.contains("alert-preview: ready"));
    assert!(text.contains("provider: pagerduty"));
    assert!(text.contains("endpoint: https://events.pagerduty.com"));
    assert!(text.contains("<redacted>"));
    assert!(!text.contains("pager-secret"));
    Ok(())
}

#[test]
fn alert_preview_rejects_datadog_target_without_api_key() {
    let error = action_from_args([
        "neo-nexus",
        "--alert-preview",
        "datadog",
        "https://event-management-intake.datadoghq.com/api/v2/events",
        "warning",
        "missing",
        "key",
    ])
    .expect_err("expected datadog api_key validation failure");

    assert!(error.to_string().contains("api_key"));
}
