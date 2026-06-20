use crate::{app::NeoNexusApp, types::NodeStatus};

#[derive(Debug, Clone, Copy)]
pub(super) struct NodeActionState {
    pub can_start: bool,
    pub can_stop: bool,
    pub can_restart: bool,
}

impl NodeActionState {
    pub fn from_app(app: &NeoNexusApp) -> Self {
        let status = app.selected_node().map(|node| node.status);
        let can_stop = status == Some(NodeStatus::Running);
        let can_start = status.is_some() && !can_stop;

        Self {
            can_start,
            can_stop,
            can_restart: can_stop,
        }
    }
}
