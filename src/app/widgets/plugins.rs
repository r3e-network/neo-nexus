use crate::app::domain::{PluginId, PluginState};

pub(in crate::app) fn plugin_enabled(states: &[PluginState], plugin_id: PluginId) -> bool {
    states
        .iter()
        .find(|state| state.plugin_id == plugin_id)
        .is_some_and(|state| state.enabled)
}
