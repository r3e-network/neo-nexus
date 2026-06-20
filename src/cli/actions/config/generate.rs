use super::{node_args::NodeConfigCliSpec, *};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::cli) struct GeneratedNodeConfigReport {
    pub(in crate::cli) node_type: NodeType,
    pub(in crate::cli) network: Network,
    pub(in crate::cli) storage_engine: StorageEngine,
    pub(in crate::cli) rpc_port: u16,
    pub(in crate::cli) p2p_port: u16,
    pub(in crate::cli) format: ConfigFormat,
    pub(in crate::cli) path: PathBuf,
    pub(in crate::cli) bytes_written: usize,
    pub(in crate::cli) validation: ConfigValidationReport,
}

impl GeneratedNodeConfigReport {
    pub(in crate::cli) fn status_label(&self) -> &'static str {
        self.validation.status_label()
    }

    fn exit_code(&self) -> i32 {
        self.validation.exit_code()
    }

    fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!("node-config-generation: {}", self.status_label()),
            format!("runtime: {}", self.node_type),
            format!("format: {}", self.format.label()),
            format!("network: {}", self.network),
            format!("storage: {}", self.storage_engine),
            format!("rpc-port: {}", self.rpc_port),
            format!("p2p-port: {}", self.p2p_port),
            format!("path: {}", self.path.display()),
            format!("bytes-written: {}", self.bytes_written),
            format!("validation: {}", self.validation.summary()),
        ];

        let mut finding_count = 0;
        for severity in [
            ConfigValidationSeverity::Critical,
            ConfigValidationSeverity::Warning,
        ] {
            for check in self
                .validation
                .checks
                .iter()
                .filter(|check| check.severity == severity)
            {
                finding_count += 1;
                lines.push(format!(
                    "finding: {} | {} | {}",
                    severity.label(),
                    check.title,
                    check.detail
                ));
            }
        }

        if finding_count == 0 {
            lines.push("finding: none".to_string());
        }
        lines.push(String::new());
        lines.join("\n")
    }
}

pub(in crate::cli::actions) fn generate_node_config_action(args: &[String]) -> Result<CliAction> {
    let generation = generate_node_config(args, "--generate-node-config")?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: generation.exit_code(),
        text: generation.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn generate_node_config_json_action(
    args: &[String],
) -> Result<CliAction> {
    let generation = generate_node_config(args, "--generate-node-config-json")?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: generation.exit_code(),
        text: generated_node_config_json_text(&generation)?,
    })
}

fn generate_node_config(args: &[String], option: &str) -> Result<GeneratedNodeConfigReport> {
    require_arg_count(args, 8, option)?;
    let spec = NodeConfigCliSpec::from_args(args)?;
    let node = spec.generated_node();
    let output_path = PathBuf::from(&args[7]);
    if output_path.is_dir() {
        anyhow::bail!(
            "node config output {} is a directory; pass a file path",
            output_path.display()
        );
    }

    let export = ConfigExporter::write_node_config_to_path(&output_path, &node, &[])
        .with_context(|| format!("failed to generate {} config", node.node_type))?;
    let text = fs::read_to_string(&export.path)
        .with_context(|| format!("failed to read generated config {}", export.path.display()))?;
    let format = ConfigFormat::for_node_type(spec.node_type);
    let validation = ConfigValidator::validate_text(&node, format, &text);

    Ok(GeneratedNodeConfigReport {
        node_type: spec.node_type,
        network: spec.network,
        storage_engine: spec.storage_engine,
        rpc_port: spec.rpc_port,
        p2p_port: spec.p2p_port,
        format,
        path: export.path,
        bytes_written: export.bytes_written,
        validation,
    })
}
