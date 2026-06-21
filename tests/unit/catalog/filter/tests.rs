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
