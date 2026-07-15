use crate::app::domain::{NodeType, RuntimePlatform};

use super::super::super::super::NeoNexusApp;

impl NeoNexusApp {
    pub(super) fn use_latest_runtime_release_for_selected_node(&mut self) {
        let platform = RuntimePlatform::current();
        let runtime = self
            .selected_node()
            .map_or(NodeType::NeoRs, |node| node.node_type);
        let latest_id = self
            .runtime_catalog
            .as_ref()
            .and_then(|catalog| catalog.latest_for(runtime, &platform))
            .map(|release| release.id.clone());

        if let Some(id) = latest_id {
            self.selected_runtime_release = Some(id);
            self.load_selected_runtime_release_into_draft();
        } else {
            self.session.notice = Some("No compatible catalog release for this runtime".to_string());
        }
    }
}
