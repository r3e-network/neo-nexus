use super::RequiredTable;

pub(super) const EVENTS_HEALTH_TABLES: &[RequiredTable] = &[
    RequiredTable {
        name: "runtime_events",
        columns: &[
            "id",
            "occurred_at_unix",
            "node_id",
            "node_name",
            "kind",
            "severity",
            "message",
        ],
    },
    RequiredTable {
        name: "alert_deliveries",
        columns: &[
            "id",
            "event_id",
            "attempted_at_unix",
            "route_label",
            "target",
            "status",
            "http_status",
            "message",
        ],
    },
    RequiredTable {
        name: "rpc_health_checks",
        columns: &[
            "id",
            "checked_at_unix",
            "node_id",
            "node_name",
            "endpoint",
            "status",
            "version",
            "block_count",
            "message",
        ],
    },
];
