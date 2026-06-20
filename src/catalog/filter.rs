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
mod tests {
    use crate::{
        catalog::{PluginCatalog, PluginId, PluginState},
        types::NodeType,
    };

    use super::*;

    #[test]
    fn plugin_filter_matches_operator_fields() {
        let catalog = PluginCatalog;
        let plugins = catalog
            .for_node_type(NodeType::NeoCli)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let states = [PluginState {
            plugin_id: PluginId::RpcServer,
            enabled: true,
        }];

        assert_ids(
            &plugins,
            &states,
            PluginDefinitionFilter::new(None, None, "json-rpc"),
            &[PluginId::RpcServer],
        );
        assert_ids(
            &plugins,
            &states,
            PluginDefinitionFilter::new(None, None, "indexing"),
            &[PluginId::ApplicationLogs, PluginId::TokensTracker],
        );
        assert_ids(
            &plugins,
            &states,
            PluginDefinitionFilter::new(None, None, "restart required"),
            &[
                PluginId::RpcServer,
                PluginId::RestServer,
                PluginId::ApplicationLogs,
                PluginId::StateService,
                PluginId::DBFTPlugin,
                PluginId::TokensTracker,
                PluginId::LevelDbStore,
                PluginId::RocksDbStore,
            ],
        );
    }

    #[test]
    fn plugin_filter_combines_state_category_and_query() {
        let catalog = PluginCatalog;
        let plugins = catalog
            .for_node_type(NodeType::NeoCli)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let states = [
            PluginState {
                plugin_id: PluginId::RpcServer,
                enabled: true,
            },
            PluginState {
                plugin_id: PluginId::RestServer,
                enabled: false,
            },
        ];
        let filter = PluginDefinitionFilter::new(Some(true), Some(PluginCategory::Api), "rpc");

        assert_ids(&plugins, &states, filter, &[PluginId::RpcServer]);
    }

    fn assert_ids(
        plugins: &[PluginDefinition],
        states: &[PluginState],
        filter: PluginDefinitionFilter,
        expected: &[PluginId],
    ) {
        let actual = filter_plugin_definitions(plugins, states, &filter)
            .iter()
            .map(|plugin| plugin.id)
            .collect::<Vec<_>>();
        assert_eq!(actual.as_slice(), expected);
    }
}
