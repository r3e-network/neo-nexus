use std::path::PathBuf;

use serde::Serialize;

mod text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigExport {
    pub path: PathBuf,
    /// Plugin configuration files written beside the primary one. Empty for
    /// neo-go and neo-rs, which configure every service in a single file.
    pub sidecar_paths: Vec<PathBuf>,
    /// Total bytes across the primary config and every sidecar.
    pub bytes_written: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceConfigExport {
    pub output_dir: PathBuf,
    pub text_path: PathBuf,
    pub json_path: PathBuf,
    pub report: WorkspaceConfigReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceConfigReport {
    pub schema_version: u32,
    pub status: &'static str,
    pub application: &'static str,
    pub application_version: String,
    pub generated_at_unix: u64,
    pub database: String,
    pub output_dir: String,
    pub node_count: usize,
    pub exported_file_count: usize,
    pub total_bytes_written: usize,
    pub nodes: Vec<NodeConfigExportReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeConfigExportReport {
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub network: String,
    pub storage_engine: String,
    pub runtime_version: String,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
    pub config_format: String,
    pub path: String,
    pub bytes_written: usize,
    pub plugin_count: usize,
    pub enabled_plugin_count: usize,
}
