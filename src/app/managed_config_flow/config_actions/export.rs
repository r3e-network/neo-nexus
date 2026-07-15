use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn export_selected_config(&mut self) {
        let Some(node) = selected_node_or_notice(self, "Select a node before exporting config")
        else {
            return;
        };

        let plugins = plugin_states_for(self, &node);
        match ConfigExporter::write_node_config(self.config_export_dir(), &node, &plugins) {
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
