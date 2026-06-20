use crate::types::{NodeConfig, NodeType};

use super::format::{config_format, ConfigFormat, RenderedConfig, RuntimeConfigProfile};

mod checks;
mod model;
mod runtimes;

use self::runtimes::{validate_neo_cli_config, validate_neo_go_config, validate_neo_rs_config};

pub use self::model::{ConfigValidationCheck, ConfigValidationReport, ConfigValidationSeverity};

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate_rendered(
        node: &NodeConfig,
        rendered: &RenderedConfig,
    ) -> ConfigValidationReport {
        Self::validate_text(node, rendered.format, &rendered.text)
    }

    pub fn validate_rendered_with_profile(
        node: &NodeConfig,
        rendered: &RenderedConfig,
        profile: Option<&RuntimeConfigProfile>,
    ) -> ConfigValidationReport {
        Self::validate_text_with_profile(node, rendered.format, &rendered.text, profile)
    }

    pub fn validate_text(
        node: &NodeConfig,
        format: ConfigFormat,
        text: &str,
    ) -> ConfigValidationReport {
        Self::validate_text_with_profile(node, format, text, None)
    }

    pub fn validate_text_with_profile(
        node: &NodeConfig,
        format: ConfigFormat,
        text: &str,
        profile: Option<&RuntimeConfigProfile>,
    ) -> ConfigValidationReport {
        let mut report = ConfigValidationReport {
            node_type: node.node_type,
            format,
            checks: Vec::new(),
        };

        let expected_format = config_format(node.node_type);
        if format != expected_format {
            report.critical(
                "Format",
                format!(
                    "{} nodes require {} config, but {} was supplied.",
                    node.node_type,
                    expected_format.label(),
                    format.label()
                ),
            );
            return report;
        }
        report.pass(
            "Format",
            format!("{} config uses {}.", node.node_type, format.label()),
        );

        if text.trim().is_empty() {
            report.critical("Content", "Generated config is empty.");
            return report;
        }

        match node.node_type {
            NodeType::NeoCli => validate_neo_cli_config(node, text, profile, &mut report),
            NodeType::NeoGo => validate_neo_go_config(node, text, profile, &mut report),
            NodeType::NeoRs => validate_neo_rs_config(node, text, profile, &mut report),
        }

        report
    }
}
