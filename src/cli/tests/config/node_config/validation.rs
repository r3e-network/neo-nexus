use super::super::super::*;

#[test]
fn node_config_validation_cli_rejects_invalid_expected_ports() {
    let zero = action_from_args([
        "neo-nexus",
        "--validate-node-config",
        "neo-rs",
        "testnet",
        "rocksdb",
        "0",
        "10333",
        "missing.toml",
    ])
    .expect_err("zero RPC port should be rejected before file access");
    assert!(zero.to_string().contains("RPC port must be greater than 0"));

    let duplicate = action_from_args([
        "neo-nexus",
        "--validate-node-config-json",
        "neo-rs",
        "testnet",
        "rocksdb",
        "10332",
        "10332",
        "missing.toml",
    ])
    .expect_err("duplicate ports should be rejected before file access");
    assert!(duplicate
        .to_string()
        .contains("RPC and P2P ports must be different"));
}

#[test]
fn node_config_validation_cli_reports_ready_neo_rs_toml() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let node = neo_rs_config_validation_node();
    let rendered = ConfigGenerator::render_for_node(&node, &[])?;
    let config_path = temp_dir.path().join("neo-rs-config.toml");
    std::fs::write(&config_path, rendered.text)?;
    let config_arg = config_path.display().to_string();

    let action = action_from_args([
        "neo-nexus",
        "--validate-node-config",
        "neo-rs",
        "testnet",
        "rocksdb",
        "20332",
        "20333",
        &config_arg,
    ])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected node config validation action");
    };

    assert_eq!(exit_code, 0);
    assert!(text.contains("node-config-validation: ready"));
    assert!(text.contains("runtime: neo-rs"));
    assert!(text.contains("format: TOML"));
    assert!(text.contains("finding: none"));
    Ok(())
}

#[test]
fn node_config_validation_json_cli_reports_ready_neo_rs_toml() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let node = neo_rs_config_validation_node();
    let rendered = ConfigGenerator::render_for_node(&node, &[])?;
    let config_path = temp_dir.path().join("neo-rs-config.toml");
    std::fs::write(&config_path, rendered.text)?;
    let config_arg = config_path.display().to_string();

    let action = action_from_args([
        "neo-nexus",
        "--validate-node-config-json",
        "neo-rs",
        "testnet",
        "rocksdb",
        "20332",
        "20333",
        &config_arg,
    ])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected node config validation JSON action");
    };

    assert_eq!(exit_code, 0);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ready");
    assert_eq!(value["success"], true);
    assert_eq!(value["report"]["node_type"], "neo-rs");
    assert_eq!(value["report"]["format"], "toml");
    assert_eq!(
        value["report"]["checks"][0]["severity"],
        serde_json::Value::String("pass".to_string())
    );
    Ok(())
}

#[test]
fn node_config_validation_cli_flags_tampered_neo_rs_ports() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let node = neo_rs_config_validation_node();
    let rendered = ConfigGenerator::render_for_node(&node, &[])?;
    let tampered = rendered.text.replace("port = 20332", "port = 30332");
    let config_path = temp_dir.path().join("neo-rs-config.toml");
    std::fs::write(&config_path, tampered)?;
    let config_arg = config_path.display().to_string();

    let action = action_from_args([
        "neo-nexus",
        "--validate-node-config",
        "neo-rs",
        "testnet",
        "rocksdb",
        "20332",
        "20333",
        &config_arg,
    ])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected node config validation action");
    };

    assert_eq!(exit_code, 1);
    assert!(text.contains("node-config-validation: invalid"));
    assert!(text.contains("finding: critical | RPC port"));
    assert!(text.contains("expected 20332"));
    Ok(())
}
