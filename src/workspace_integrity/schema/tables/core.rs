use super::RequiredTable;

pub(super) const CORE_TABLES: &[RequiredTable] = &[
    RequiredTable {
        name: "nodes",
        columns: &[
            "id",
            "name",
            "node_type",
            "network",
            "binary_path",
            "args",
            "runtime_version",
            "storage_engine",
            "rpc_port",
            "p2p_port",
            "ws_port",
            "status",
            "pid",
        ],
    },
    RequiredTable {
        name: "plugin_states",
        columns: &["node_id", "plugin_id", "enabled"],
    },
    RequiredTable {
        name: "plugin_installations",
        columns: &[
            "node_id",
            "plugin_id",
            "installed_path",
            "manifest_path",
            "source_path",
            "sha256",
            "package_bytes",
            "installed_files",
            "expanded_bytes",
            "installed_at_unix",
        ],
    },
    RequiredTable {
        name: "workspace_settings",
        columns: &["key", "value"],
    },
];
