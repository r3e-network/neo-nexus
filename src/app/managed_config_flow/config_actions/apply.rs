use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn apply_selected_managed_config(&mut self) {
        let Some(node) = selected_node_or_notice(self, "Select a node before applying config")
        else {
            return;
        };

        let plugins = plugin_states_for(self, &node);
        let path = self.managed_config_path(&node);
        match ConfigExporter::write_node_config_to_path(&path, &node, &plugins) {
            Ok(export) => {
                record_managed_config_applied(self, &node, &export, node_requires_restart(&node));
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
