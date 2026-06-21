use super::*;

#[test]
fn alert_payload_contains_stable_event_fields() {
    let event = event(EventSeverity::Critical);
    let payload = alert_webhook_payload(&event, "2.5.3");

    assert_eq!(payload["schema_version"], 1);
    assert_eq!(payload["application"], "NeoNexus");
    assert_eq!(payload["application_version"], "2.5.3");
    assert_eq!(payload["event"]["id"], 7);
    assert_eq!(payload["event"]["kind"], "rpc-health-checked");
    assert_eq!(payload["event"]["severity"], "critical");
    assert_eq!(payload["event"]["node_name"], "validator");
}

#[test]
fn provider_payloads_match_slack_discord_telegram_pagerduty_opsgenie_and_datadog_shapes() {
    let event = event(EventSeverity::Critical);
    let slack = alert_provider_payload(AlertProvider::Slack, &event, "2.5.3");
    let discord = alert_provider_payload(AlertProvider::Discord, &event, "2.5.3");
    let telegram = telegram_alert_payload(&event, "2.5.3", "@neo_ops");
    let pagerduty = pagerduty_alert_payload(&event, "2.5.3", "routing-key-123");
    let opsgenie = opsgenie_alert_payload(&event, "2.5.3");
    let datadog = datadog_event_payload(&event, "2.5.3");

    assert!(slack["text"]
        .as_str()
        .is_some_and(|text| text.contains("RPC health")));
    assert_eq!(slack["blocks"][0]["type"], "header");
    assert_eq!(slack["blocks"][1]["text"]["type"], "mrkdwn");

    assert_eq!(discord["username"], "NeoNexus");
    assert!(discord["content"]
        .as_str()
        .is_some_and(|text| text.contains("critical")));
    assert_eq!(discord["embeds"][0]["color"], 0xdc2626);
    assert_eq!(discord["embeds"][0]["fields"][0]["name"], "Kind");

    assert_eq!(telegram["chat_id"], "@neo_ops");
    assert_eq!(telegram["parse_mode"], "HTML");
    assert!(telegram["text"]
        .as_str()
        .is_some_and(|text| text.contains("<b>NeoNexus")));
    assert!(telegram["text"]
        .as_str()
        .is_some_and(|text| text.contains("RPC health")));

    assert_eq!(pagerduty["routing_key"], "routing-key-123");
    assert_eq!(pagerduty["event_action"], "trigger");
    assert_eq!(pagerduty["payload"]["severity"], "critical");
    assert_eq!(pagerduty["payload"]["source"], "NeoNexus/validator");
    assert!(pagerduty["payload"]["summary"]
        .as_str()
        .is_some_and(|summary| summary.contains("RPC health")));
    assert_eq!(pagerduty["custom_details"]["application_version"], "2.5.3");

    assert!(opsgenie["message"]
        .as_str()
        .is_some_and(|message| message.contains("RPC health")));
    assert_eq!(opsgenie["alias"], "neonexus:rpc-health-checked:node-1");
    assert_eq!(opsgenie["source"], "NeoNexus/validator");
    assert_eq!(opsgenie["priority"], "P1");
    assert_eq!(opsgenie["details"]["application_version"], "2.5.3");
    assert_eq!(opsgenie["details"]["event_id"], "7");

    assert_eq!(datadog["data"]["type"], "event");
    assert_eq!(datadog["data"]["attributes"]["category"], "alert");
    assert_eq!(
        datadog["data"]["attributes"]["aggregation_key"],
        "neonexus:rpc-health-checked:node-1"
    );
    assert_eq!(
        datadog["data"]["attributes"]["attributes"]["status"],
        "error"
    );
    assert_eq!(datadog["data"]["attributes"]["attributes"]["priority"], "1");
    assert!(datadog["data"]["attributes"]["tags"]
        .as_array()
        .is_some_and(|tags| tags.iter().any(|tag| tag == "service:neo-nexus")));
    assert_eq!(
        datadog["data"]["attributes"]["attributes"]["custom"]["application_version"],
        "2.5.3"
    );
}
