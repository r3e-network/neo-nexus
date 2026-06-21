pub use crate::{
    alerts, backup, catalog, config, dashboard, diagnostics, event_journal_report, events,
    federation, launch, logs, metrics, plugins, port_planner, preflight, private_network,
    readiness_report, redaction, release_pack, repository, roles, rpc_health, runtime,
    runtime_smoke, snapshots, supervisor, support_bundle, types, wallet, watchdog,
    workspace_integrity,
};

#[cfg(test)]
mod tests {
    #[test]
    fn core_exposes_domain_services_without_surface_modules() {
        assert_eq!(crate::core::types::NodeStatus::Running.label(), "Running");
        assert!(std::any::type_name::<crate::core::repository::Repository>().contains("repository"));
    }
}
