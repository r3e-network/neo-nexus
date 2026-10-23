//! Controls from the browser. Node lifecycle delegates to the supervision
//! engine, so a start from the page and a start from the watchdog are the same
//! code path against the same supervisor; policy forms do the same for settings.
//! Handlers answer with a redirect carrying a flash message, so every control is
//! a plain form post that works without JavaScript.

mod maintenance;
mod node;
mod settings;

mod artifacts;

use axum::response::{IntoResponse, Response};

pub use artifacts::{
    check_workspace_integrity, export_readiness_report, export_support_bundle,
    reconcile_node_config,
};
pub use maintenance::{
    apply_snapshot, clear_logs, handle_backup_export, handle_backup_import, BackupImportForm,
    ClearLogsForm,
};
pub use node::{
    batch_node_action, node_restart, node_start, node_stop, smoke_test_node, toggle_restart_hold,
};
pub use settings::{
    preview_alert_routing, save_alert_routing, save_density, save_federation_monitor,
    save_rpc_health_monitor, save_runtime_upgrade_policy, save_watchdog, AlertRoutingForm,
    DensityForm, MonitorForm, RuntimeUpgradeForm, WatchdogForm,
};

/// Run blocking work off the async executor.
///
/// Exporting a support bundle, writing a backup archive, clearing the log
/// directory and hashing a workspace all take file-system time that has no
/// await point in it. Calling them straight from an `async fn` pins a Tokio
/// worker thread for the whole operation, and the server serves every other
/// request from a fixed pool of those threads — so one operator pressing
/// "Export bundle" could stall the console for everyone else while it ran.
///
/// The work moves to the blocking pool and only the response comes back.
pub(crate) async fn blocking<F>(work: F) -> Response
where
    F: FnOnce() -> Response + Send + 'static,
{
    match tokio::task::spawn_blocking(work).await {
        Ok(response) => response,
        // The only way to get here is a panic inside the operation. Say so
        // rather than implying the request did nothing.
        Err(error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("the operation did not complete: {error}"),
        )
            .into_response(),
    }
}
