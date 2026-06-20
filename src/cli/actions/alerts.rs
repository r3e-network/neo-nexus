use super::*;

pub(in crate::cli) fn alert_preview_text(args: &[String]) -> Result<String> {
    Ok(alert_preview_report(args)?.to_cli_text())
}

pub(in crate::cli) fn alert_preview_json_action(args: &[String]) -> Result<CliAction> {
    Ok(CliAction::Print(alert_preview_json_text(
        &alert_preview_report(args)?,
    )?))
}

fn alert_preview_report(args: &[String]) -> Result<AlertPreviewReport> {
    if args.len() < 6 {
        anyhow::bail!(
            "{} requires <provider> <target-url> <info|warning|critical> <message...>",
            args.get(1).map_or("--alert-preview", String::as_str)
        );
    }

    let provider = args[2].parse::<AlertProvider>()?;
    let target_url = &args[3];
    let severity = args[4].parse::<EventSeverity>()?;
    let message = args[5..].join(" ");
    let event = RuntimeEvent {
        id: 0,
        occurred_at_unix: current_unix_time()?,
        node_id: None,
        node_name: Some("alert-preview".to_string()),
        kind: EventKind::AlertRoutingPolicyUpdated,
        severity,
        message,
    };

    preview_alert_route(provider, target_url, &event, env!("CARGO_PKG_VERSION"))
}
