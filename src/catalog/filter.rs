use super::{PluginCategory, PluginDefinition, PluginId, PluginState};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PluginDefinitionFilter {
    pub enabled: Option<bool>,
    pub category: Option<PluginCategory>,
    pub query: String,
}

impl PluginDefinitionFilter {
    pub fn new(
        enabled: Option<bool>,
        category: Option<PluginCategory>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            enabled,
            category,
            query: query.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.category.is_none() && self.query.trim().is_empty()
    }
}

pub fn filter_plugin_definitions(
    plugins: &[PluginDefinition],
    states: &[PluginState],
    filter: &PluginDefinitionFilter,
) -> Vec<PluginDefinition> {
    let query = filter.query.trim().to_lowercase();
    plugins
        .iter()
        .filter(|plugin| {
            filter
                .category
                .is_none_or(|category| plugin.category == category)
        })
        .filter(|plugin| {
            filter
                .enabled
                .is_none_or(|enabled| plugin_is_enabled(states, plugin.id) == enabled)
        })
        .filter(|plugin| query.is_empty() || plugin_matches(plugin, &query))
        .cloned()
        .collect()
}

fn plugin_is_enabled(states: &[PluginState], plugin_id: PluginId) -> bool {
    states
        .iter()
        .find(|state| state.plugin_id == plugin_id)
        .is_some_and(|state| state.enabled)
}

fn plugin_matches(plugin: &PluginDefinition, query: &str) -> bool {
    text_matches(&plugin.id.to_string(), query)
        || text_matches(plugin.name, query)
        || text_matches(&plugin.category.to_string(), query)
        || text_matches(plugin.description, query)
        || plugin
            .node_types
            .iter()
            .any(|node_type| text_matches(&node_type.to_string(), query))
        || text_matches(restart_label(plugin.requires_restart), query)
}

fn restart_label(requires_restart: bool) -> &'static str {
    if requires_restart {
        "restart required"
    } else {
        "live reload"
    }
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
#[path = "../../tests/unit/catalog/filter/tests.rs"]
mod tests;
