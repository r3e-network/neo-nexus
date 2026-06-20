use crate::{catalog::PluginId, roles::RolePluginChange};

pub(super) fn enable(plugin_id: PluginId, reason: &'static str) -> RolePluginChange {
    RolePluginChange {
        plugin_id,
        enabled: true,
        reason,
    }
}

pub(super) fn disable(plugin_id: PluginId, reason: &'static str) -> RolePluginChange {
    RolePluginChange {
        plugin_id,
        enabled: false,
        reason,
    }
}
