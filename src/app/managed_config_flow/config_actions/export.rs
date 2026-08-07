use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn export_selected_config(&mut self) {
        let Some(node) = selected_node_or_notice(self, "Select a node before exporting config")
        else {
            return;
        };

        let plugins = plugin_states_for(self, &node);
        let context = generation_context_for(self, &node);
        let path = ConfigExporter::target_path(self.config_export_dir(), &node);
        match ConfigExporter::write_node_config_to_path_with_context(
            &path, &node, &plugins, None, &context,
        ) {
            Ok(export) => {
                self.session.notice = Some(format!(
                    "Config exported: {} ({} bytes)",
                    short_path(&export.path, 54),
                    export.bytes_written
                ));
                self.record_node_event(
                    &node,
                    EventKind::ConfigExported,
                    EventSeverity::Info,
                    format!("Config exported to {}", export.path.display()),
                );
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
