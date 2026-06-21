pub mod distribution;
pub mod node;
pub mod operations;
pub mod runtime;
pub mod security;
pub mod workspace;

#[cfg(test)]
mod tests {
    #[test]
    fn core_facade_groups_reusable_domain_services_by_operator_boundary() {
        assert_eq!(crate::core::node::NodeStatus::Running.label(), "Running");
        assert!(std::any::type_name::<crate::core::node::NodeConfig>().contains("types"));
        assert!(std::any::type_name::<crate::core::workspace::Repository>().contains("repository"));
        assert!(std::any::type_name::<crate::core::runtime::RuntimeRelease>().contains("runtime"));
        assert!(
            std::any::type_name::<crate::core::operations::FleetDiagnostics>()
                .contains("diagnostics")
        );
        assert!(
            std::any::type_name::<crate::core::distribution::ReleasePackager>()
                .contains("release_pack")
        );
        assert_eq!(
            crate::core::security::redact_sensitive_text("api_key=secret"),
            "api_key=<redacted>"
        );
    }
}
