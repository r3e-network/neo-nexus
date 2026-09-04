use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantDraft {
    pub id: String,
    pub name: String,
    pub agent_id: String,
    pub node_ids: Vec<String>,
    pub all_nodes: bool,
    pub can_operate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantProfile {
    pub id: String,
    pub name: String,
    pub agent_id: String,
    pub node_ids: Vec<String>,
    pub all_nodes: bool,
    pub can_operate: bool,
    pub enabled: bool,
}

impl AssistantProfile {
    pub fn allows_node(&self, id: &str) -> bool {
        self.enabled && (self.all_nodes || self.node_ids.iter().any(|node| node == id))
    }
}
