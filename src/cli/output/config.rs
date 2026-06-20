use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::{
    config::{ConfigFormat, ConfigValidationReport},
    types::NodeType,
};

use super::{super::actions::GeneratedNodeConfigReport, json_text};

#[derive(Debug, Serialize)]
struct GeneratedNodeConfigJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    generation: GeneratedNodeConfigJson<'a>,
}

#[derive(Debug, Serialize)]
struct GeneratedNodeConfigJson<'a> {
    node_type: NodeType,
    network: String,
    storage_engine: String,
    rpc_port: u16,
    p2p_port: u16,
    format: ConfigFormat,
    path: String,
    bytes_written: usize,
    validation: GeneratedNodeConfigValidationJson<'a>,
}

#[derive(Debug, Serialize)]
struct GeneratedNodeConfigValidationJson<'a> {
    status: &'static str,
    success: bool,
    summary: String,
    report: &'a ConfigValidationReport,
}

#[derive(Debug, Serialize)]
struct NodeConfigValidationJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    source: String,
    report: &'a ConfigValidationReport,
}

pub(in crate::cli) fn generated_node_config_json_text(
    generation: &GeneratedNodeConfigReport,
) -> Result<String> {
    json_text(&GeneratedNodeConfigJsonReport {
        schema_version: 1,
        status: generation.status_label(),
        success: generation.validation.is_success(),
        generation: GeneratedNodeConfigJson {
            node_type: generation.node_type,
            network: generation.network.to_string(),
            storage_engine: generation.storage_engine.to_string(),
            rpc_port: generation.rpc_port,
            p2p_port: generation.p2p_port,
            format: generation.format,
            path: generation.path.display().to_string(),
            bytes_written: generation.bytes_written,
            validation: GeneratedNodeConfigValidationJson {
                status: generation.validation.status_label(),
                success: generation.validation.is_success(),
                summary: generation.validation.summary(),
                report: &generation.validation,
            },
        },
    })
}

pub(in crate::cli) fn node_config_validation_json_text(
    source_path: &Path,
    report: &ConfigValidationReport,
) -> Result<String> {
    json_text(&NodeConfigValidationJsonReport {
        schema_version: 1,
        status: report.status_label(),
        success: report.is_success(),
        source: source_path.display().to_string(),
        report,
    })
}
