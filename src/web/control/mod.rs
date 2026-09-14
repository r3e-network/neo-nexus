//! Controls from the browser. Node lifecycle delegates to the supervision
//! engine, so a start from the page and a start from the watchdog are the same
//! code path against the same supervisor; policy forms do the same for settings.
//! Handlers answer with a redirect carrying a flash message, so every control is
//! a plain form post that works without JavaScript.

mod maintenance;
mod node;
mod settings;

pub use maintenance::{
    apply_snapshot, clear_logs, handle_backup_export, handle_backup_import, BackupImportForm,
};
pub use node::{batch_node_action, node_restart, node_start, node_stop, smoke_test_node};
pub use settings::{
    save_alert_routing, save_density, save_federation_monitor, save_rpc_health_monitor,
    save_runtime_upgrade_policy, save_watchdog, AlertRoutingForm, DensityForm, MonitorForm,
    RuntimeUpgradeForm, WatchdogForm,
};
