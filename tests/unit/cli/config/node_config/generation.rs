use super::super::super::*;

#[test]
fn node_config_generation_cli_writes_ready_neo_rs_toml() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("generated-neo-rs.toml");
    let config_arg = config_path.display().to_string();

    let action = action_from_args([
        "neo-nexus",
        "--generate-node-config",
        "neo-rs",
        "testnet",
        "rocksdb",
        "20332",
        "20333",
        &config_arg,
    ])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected node config generation action");
    };

    assert_eq!(exit_code, 0);
    assert!(config_path.is_file());
    let generated = std::fs::read_to_string(&config_path)?;
    let parsed: toml::Value = toml::from_str(&generated)?;
    assert_eq!(parsed["network"]["network_type"].as_str(), Some("testnet"));
    assert_eq!(parsed["rpc"]["port"].as_integer(), Some(20332));
    assert_eq!(parsed["p2p"]["port"].as_integer(), Some(20333));
    assert!(text.contains("node-config-generation: ready"));
    assert!(text.contains("runtime: neo-rs"));
    assert!(text.contains("format: TOML"));
    assert!(text.contains("validation:"));
    assert!(text.contains("finding: none"));
    Ok(())
}

#[test]
fn node_config_generation_json_cli_reports_path_and_validation() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("generated-neo-rs.toml");
    let config_arg = config_path.display().to_string();

    let action = action_from_args([
        "neo-nexus",
        "--generate-node-config-json",
        "neo-rs",
        "testnet",
        "rocksdb",
        "20332",
        "20333",
        &config_arg,
    ])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected node config generation JSON action");
    };

    assert_eq!(exit_code, 0);
    assert!(config_path.is_file());
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ready");
    assert_eq!(value["success"], true);
    assert_eq!(value["generation"]["node_type"], "neo-rs");
    assert_eq!(value["generation"]["format"], "toml");
    assert_eq!(value["generation"]["path"], config_arg);
    assert_eq!(value["generation"]["validation"]["status"], "ready");
    assert_eq!(
        value["generation"]["validation"]["report"]["node_type"],
        "neo-rs"
    );
    Ok(())
}
