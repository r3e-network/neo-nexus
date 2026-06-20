use super::RequiredIndex;

pub(in crate::workspace_integrity) const REQUIRED_INDEXES: &[RequiredIndex] = &[
    RequiredIndex {
        table: "runtime_events",
        name: "idx_runtime_events_recent",
    },
    RequiredIndex {
        table: "alert_deliveries",
        name: "idx_alert_deliveries_recent",
    },
    RequiredIndex {
        table: "rpc_health_checks",
        name: "idx_rpc_health_checks_node_recent",
    },
    RequiredIndex {
        table: "remote_servers",
        name: "idx_remote_servers_enabled",
    },
    RequiredIndex {
        table: "remote_server_probe_records",
        name: "idx_remote_server_probe_records_recent",
    },
];
