use anyhow::Result;
use serde::Serialize;

mod alerts;
mod backup;
mod config;
mod health;
mod launch;
mod quality;
mod release;
mod wallet;

pub(super) use self::{
    alerts::alert_preview_json_text,
    backup::{backup_export_json_text, backup_import_json_text, backup_validation_json_text},
    config::{generated_node_config_json_text, node_config_validation_json_text},
    health::{
        rpc_health_json_text, runtime_smoke_json_text, workspace_metrics_json_text,
        workspace_readiness_exit_code, workspace_readiness_json_text, workspace_readiness_text,
    },
    launch::launch_pack_sidecars_json_text,
    quality::{
        ci_policy_json_text, native_ui_audit_json_text, source_purity_json_text,
        source_quality_json_text,
    },
    release::{
        release_package_verification_failure_json_text, release_package_verification_json_text,
    },
    wallet::{wallet_profile_import_text, wallet_validation_json_text},
};

fn json_text<T: Serialize>(payload: &T) -> Result<String> {
    Ok(format!("{}\n", serde_json::to_string_pretty(payload)?))
}
