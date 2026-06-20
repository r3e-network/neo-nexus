use super::{node_args::NodeConfigCliSpec, *};

pub(in crate::cli::actions) fn validate_node_config_action(args: &[String]) -> Result<CliAction> {
    let (source_path, report) = validate_node_config(args, "--validate-node-config")?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: report.to_cli_text(&source_path),
    })
}

pub(in crate::cli::actions) fn validate_node_config_json_action(
    args: &[String],
) -> Result<CliAction> {
    let (source_path, report) = validate_node_config(args, "--validate-node-config-json")?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: node_config_validation_json_text(&source_path, &report)?,
    })
}

fn validate_node_config(
    args: &[String],
    option: &str,
) -> Result<(PathBuf, ConfigValidationReport)> {
    require_arg_count(args, 8, option)?;
    let spec = NodeConfigCliSpec::from_args(args)?;
    let source_path = PathBuf::from(&args[7]);
    if !source_path.is_file() {
        anyhow::bail!(
            "node config {} does not exist; pass an existing config file",
            source_path.display()
        );
    }
    let text = fs::read_to_string(&source_path)
        .with_context(|| format!("failed to read node config {}", source_path.display()))?;
    let node = spec.validation_node();
    let report =
        ConfigValidator::validate_text(&node, ConfigFormat::for_node_type(spec.node_type), &text);
    Ok((source_path, report))
}
