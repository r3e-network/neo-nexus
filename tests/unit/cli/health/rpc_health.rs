use super::super::*;

#[test]
fn rpc_health_cli_reports_unreachable_endpoint() -> Result<()> {
    let action = action_from_args(["neo-nexus", "--rpc-health", "127.0.0.1:1"])?;

    assert!(matches!(action, CliAction::Print(text) if text.contains("rpc-health: unreachable")));
    Ok(())
}

#[test]
fn rpc_health_json_cli_reports_unreachable_endpoint() -> Result<()> {
    let action = action_from_args(["neo-nexus", "--rpc-health-json", "127.0.0.1:1"])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected RPC health JSON action");
    };

    assert_eq!(exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "unreachable");
    assert_eq!(value["success"], false);
    assert_eq!(value["report"]["endpoint"], "http://127.0.0.1:1");
    assert_eq!(value["report"]["status"], "unreachable");
    assert_eq!(value["report"]["methods"].as_array().map(Vec::len), Some(2));
    assert_eq!(value["report"]["methods"][0]["method"], "getversion");
    assert_eq!(value["report"]["methods"][0]["ok"], false);
    Ok(())
}

#[test]
fn rpc_health_cli_probes_neo_x_methods_for_neo_x_family() -> Result<()> {
    let action = action_from_args(["neo-nexus", "--rpc-health-json", "127.0.0.1:1", "neo-x"])?;
    let CliAction::PrintWithExitCode { text, .. } = action else {
        anyhow::bail!("expected RPC health JSON action");
    };

    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(
        value["report"]["methods"][0]["method"],
        "web3_clientVersion"
    );
    assert_eq!(value["report"]["methods"][1]["method"], "eth_blockNumber");
    Ok(())
}

#[test]
fn rpc_health_cli_rejects_unknown_chain_family() {
    let error = action_from_args(["neo-nexus", "--rpc-health", "127.0.0.1:1", "neo-legacy"])
        .expect_err("an unknown chain family slug must fail");
    assert!(error.to_string().contains("neo-legacy"));
    assert!(error.to_string().contains("neo-n3 or neo-x"));
}
