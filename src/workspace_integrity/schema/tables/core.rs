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
    // The duty a node performs and the wallet its signing services use. Both
    // decide which sections a generated config carries, so a workspace missing
    // them is not structurally sound — it is one whose fleet will regenerate as
    // plain relays. They self-heal on the next open, which is why their absence
    // is worth reporting rather than fatal.
    RequiredTable {
        name: "node_roles",
        columns: &["node_id", "role"],
    },
    RequiredTable {
        name: "node_wallets",
        columns: &["node_id", "wallet_profile_id"],
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
