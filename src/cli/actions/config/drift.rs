//! `--config-drift`: what changed between the workspace and the files on disk.
//!
//! Each node's managed config is re-rendered from the workspace — the SAME
//! render a launch would write — and compared with the file a node would
//! actually boot from. Three findings per node: a file that does not exist,
//! content the current expectations reject (the proxy for "a newer node
//! version no longer understands this config"), and plain line drift with
//! samples of the unexpected lines. Resolution stays with the existing
//! surfaces: re-apply the managed config, or edit and keep the drift.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

use super::*;
use crate::core::lifecycle::generation_context_for_node;
use crate::core::workspace::{
    line_drift, ConfigGenerator, ConfigValidationSeverity, ConfigValidator,
};

pub(in crate::cli::actions) fn config_drift_text(args: &[String]) -> Result<String> {
    Ok(config_drift_report(args)?.to_cli_text())
}

pub(in crate::cli::actions) fn config_drift_json_action(args: &[String]) -> Result<CliAction> {
    let report = config_drift_report(args)?;
    let attention_count = report.attention_count;
    Ok(CliAction::PrintWithExitCode {
        text: config_drift_json_text(&report)?,
        exit_code: i32::from(attention_count > 0),
    })
}

fn config_drift_report(args: &[String]) -> Result<ConfigDriftReport> {
    let option = args.get(1).map_or("--config-drift", String::as_str);
    require_arg_count(args, 3, option)?;
    let repository = Repository::open(PathBuf::from(&args[2]))
        .with_context(|| format!("failed to open workspace database {}", args[2]))?;
    let nodes = repository
        .list_nodes()
        .with_context(|| format!("failed to load nodes from {}", args[2]))?;

    // The same children convention every launch uses: managed configs live in
    // `nodes/` beside the database.
    let nodes_root = repository
        .db_path()
        .parent()
        .map_or_else(|| PathBuf::from("nodes"), |parent| parent.join("nodes"));

    let mut node_reports = Vec::new();
    for node in &nodes {
        node_reports.push(node_drift_report(&repository, &nodes_root, node));
    }
    let attention_count = node_reports
        .iter()
        .filter(|report| report.status == "attention")
        .count();
    Ok(ConfigDriftReport {
        node_count: nodes.len(),
        attention_count,
        nodes: node_reports,
    })
}

fn node_drift_report(
    repository: &Repository,
    nodes_root: &Path,
    node: &NodeConfig,
) -> DriftNodeReport {
    let managed = ConfigExporter::managed_target_path(nodes_root.join(&node.id), node);
    let mut findings = Vec::new();

    let plugins = repository.list_plugin_states(&node.id).unwrap_or_default();
    let context = generation_context_for_node(repository, node);
    let rendered = ConfigGenerator::render_for_node_with_context(node, &plugins, None, &context);
    let disk = fs::read_to_string(&managed).ok();

    match (&rendered, disk) {
        (Err(error), _) => findings.push(DriftFinding {
            kind: "generation-failed",
            detail: format!("the workspace cannot render this config: {error}"),
        }),
        (Ok(_), None) => findings.push(DriftFinding {
            kind: "missing",
            detail: format!(
                "no managed config on disk at {}; it is written on the next launch",
                managed.display()
            ),
        }),
        (Ok(rendered), Some(disk_text)) => {
            // The version-conflict proxy: the validator encodes what the
            // current expectations accept, so a key a newer node version
            // dropped or renamed shows up here as critical or warning.
            let validation = ConfigValidator::validate_text_with_context(
                node,
                rendered.format,
                &disk_text,
                None,
                &context,
            );
            for check in &validation.checks {
                if check.severity == ConfigValidationSeverity::Pass {
                    continue;
                }
                findings.push(DriftFinding {
                    kind: "validation",
                    detail: format!(
                        "{} {}: {}",
                        check.severity.label(),
                        check.title,
                        check.detail
                    ),
                });
            }

            let drift = line_drift(&rendered.text, &disk_text);
            if !drift.is_empty() {
                let mut detail = format!(
                    "{} unexpected line(s) on disk, {} missing line(s) a fresh render would write",
                    drift.unexpected_lines, drift.missing_lines
                );
                if !drift.unexpected_samples.is_empty() {
                    detail.push_str("; unexpected sample(s): ");
                    detail.push_str(&drift.unexpected_samples.join(" | "));
                }
                findings.push(DriftFinding {
                    kind: "drift",
                    detail,
                });
            }
        }
    }

    let status = if findings.is_empty() {
        "ok"
    } else {
        "attention"
    };
    DriftNodeReport {
        node_id: node.id.clone(),
        node_name: node.name.clone(),
        node_type: node.node_type.to_string(),
        config_path: managed.display().to_string(),
        status,
        findings,
    }
}

#[derive(Serialize)]
struct ConfigDriftReport {
    node_count: usize,
    attention_count: usize,
    nodes: Vec<DriftNodeReport>,
}

#[derive(Serialize)]
struct DriftNodeReport {
    node_id: String,
    node_name: String,
    node_type: String,
    config_path: String,
    status: &'static str,
    findings: Vec<DriftFinding>,
}

#[derive(Serialize)]
struct DriftFinding {
    kind: &'static str,
    detail: String,
}

impl ConfigDriftReport {
    fn to_cli_text(&self) -> String {
        let mut lines = vec![format!(
            "config-drift: {} of {} node(s) need attention",
            self.attention_count, self.node_count
        )];
        for node in &self.nodes {
            lines.push(format!(
                "node: {} [{}] — {}",
                node.node_name, node.node_type, node.status
            ));
            lines.push(format!("  config: {}", node.config_path));
            for finding in &node.findings {
                lines.push(format!("  {}: {}", finding.kind, finding.detail));
            }
        }
        lines.push(String::new());
        lines.join("\n")
    }
}

fn config_drift_json_text(report: &ConfigDriftReport) -> Result<String> {
    let value = json!({
        "schema_version": 1,
        "node_count": report.node_count,
        "attention_count": report.attention_count,
        "success": report.attention_count == 0,
        "nodes": report.nodes,
    });
    Ok(serde_json::to_string_pretty(&value)?)
}
