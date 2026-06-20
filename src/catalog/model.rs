use crate::types::NodeType;

use super::{definitions::PLUGIN_DEFINITIONS, PluginCategory, PluginId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDefinition {
    pub id: PluginId,
    pub name: &'static str,
    pub category: PluginCategory,
    pub description: &'static str,
    pub node_types: &'static [NodeType],
    pub requires_restart: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginState {
    pub plugin_id: PluginId,
    pub enabled: bool,
}

#[derive(Default)]
pub struct PluginCatalog;

impl PluginCatalog {
    pub fn all(&self) -> &'static [PluginDefinition] {
        &PLUGIN_DEFINITIONS
    }

    pub fn for_node_type(&self, node_type: NodeType) -> Vec<&'static PluginDefinition> {
        self.all()
            .iter()
            .filter(|plugin| plugin.node_types.contains(&node_type))
            .collect()
    }

    pub fn definition(&self, id: PluginId) -> Option<&'static PluginDefinition> {
        self.all().iter().find(|plugin| plugin.id == id)
    }
}
