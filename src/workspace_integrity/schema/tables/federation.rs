use super::RequiredTable;

pub(super) const FEDERATION_TABLES: &[RequiredTable] = &[
    RequiredTable {
        name: "remote_servers",
        columns: &[
            "id",
            "name",
            "base_url",
            "description",
            "enabled",
            "created_at_unix",
            "updated_at_unix",
        ],
    },
    RequiredTable {
        name: "remote_server_probe_records",
        columns: &[
            "id",
            "checked_at_unix",
            "remote_server_id",
            "remote_server_name",
            "base_url",
            "status",
            "total_nodes",
            "running_nodes",
            "syncing_nodes",
            "error_nodes",
            "total_blocks",
            "total_peers",
            "public_node_count",
            "message",
        ],
    },
];
