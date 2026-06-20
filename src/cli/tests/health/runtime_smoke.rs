use super::super::*;

#[test]
fn runtime_smoke_cli_reports_blocked_missing_binary() -> Result<()> {
    let action = action_from_args([
        "neo-nexus",
        "--runtime-smoke",
        "neo-rs",
        "/definitely/missing/neo-node",
    ])?;

    assert!(matches!(action, CliAction::Print(text) if text.contains("runtime-smoke: blocked")));
    Ok(())
}

#[test]
fn runtime_smoke_json_cli_reports_blocked_missing_binary() -> Result<()> {
    let action = action_from_args([
        "neo-nexus",
        "--runtime-smoke-json",
        "neo-rs",
        "/definitely/missing/neo-node",
    ])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected runtime smoke JSON action");
    };

    assert_eq!(exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "blocked");
    assert_eq!(value["success"], false);
    assert_eq!(value["report"]["node_type"], "neo-rs");
    assert_eq!(
        value["report"]["binary_path"],
        "/definitely/missing/neo-node"
    );
    assert_eq!(value["report"]["preflight"]["node_type"], "neo-rs");
    assert_eq!(
        value["report"]["preflight"]["resolved_path"],
        serde_json::Value::Null
    );
    assert_eq!(value["report"]["binary_evidence"]["status"], "unavailable");
    assert_eq!(
        value["report"]["binary_evidence"]["runtime_path"],
        "/definitely/missing/neo-node"
    );
    assert!(value["report"]["binary_evidence"]["message"]
        .as_str()
        .is_some_and(|message| message.contains("hash unavailable")));
    assert_eq!(
        value["report"]["preflight"]["checks"][0]["severity"],
        "critical"
    );
    assert_eq!(
        value["report"]["attempts"].as_array().map(Vec::len),
        Some(0)
    );
    Ok(())
}
