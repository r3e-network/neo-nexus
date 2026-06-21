use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn plugin_filter(&self) -> PluginDefinitionFilter {
        PluginDefinitionFilter::new(
            self.plugin_enabled_filter,
            self.plugin_category_filter,
            self.plugin_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_plugins_for_node(
        &self,
        node: &NodeConfig,
    ) -> Vec<PluginDefinition> {
        let plugins = self
            .plugin_catalog
            .for_node_type(node.node_type)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let states = self
            .repository
            .list_plugin_states(&node.id)
            .unwrap_or_default();
        filter_plugin_definitions(&plugins, &states, &self.plugin_filter())
    }

    pub(in crate::app) fn ensure_valid_plugin_selection(&mut self, node: &NodeConfig) {
        let plugins = self.filtered_plugins_for_node(node);
        if self
            .selected_plugin
            .is_none_or(|id| !plugins.iter().any(|plugin| plugin.id == id))
        {
            self.selected_plugin = plugins.first().map(|plugin| plugin.id);
        }
        self.plugin_page = clamp_page(self.plugin_page, plugins.len(), PLUGIN_PAGE_SIZE);
    }

    pub(super) fn toggle_plugin(&mut self, plugin_id: PluginId, enabled: bool) {
        let Some(node_id) = self.selected_node().map(|node| node.id.clone()) else {
            self.notice = Some("Create a node before changing plugins".to_string());
            return;
        };

        match self
            .repository
            .set_plugin_enabled(&node_id, plugin_id, enabled)
        {
            Ok(()) => {
                if let Some(node) = self.selected_node().cloned() {
                    self.record_node_event(
                        &node,
                        EventKind::PluginUpdated,
                        EventSeverity::Info,
                        format!(
                            "{plugin_id} {}",
                            if enabled { "enabled" } else { "disabled" }
                        ),
                    );
                }
                self.notice = Some(format!(
                    "{plugin_id} {}",
                    if enabled { "enabled" } else { "disabled" }
                ));
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(super) fn fill_selected_plugin_package_sha256(&mut self) {
        let source_path = PathBuf::from(self.plugin_package_source.trim());
        match sha256_file(&source_path) {
            Ok((sha256, bytes)) => {
                self.plugin_package_expected_sha256 = sha256;
                self.notice = Some(format!("Plugin package hashed: {}", format_bytes(bytes)));
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(super) fn install_selected_plugin_package(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.notice = Some("Select a node before installing a plugin package".to_string());
            return;
        };
        if node.node_type != NodeType::NeoCli {
            self.notice = Some("Plugin packages are supported for neo-cli nodes only".to_string());
            return;
        }
        if node.status.is_active() {
            self.notice =
                Some("Stop the selected node before installing a plugin package".to_string());
            return;
        }
        let Some(plugin_id) = self.selected_plugin else {
            self.notice = Some("Select a plugin before installing a package".to_string());
            return;
        };

        let label = self
            .plugin_catalog
            .definition(plugin_id)
            .map_or_else(|| plugin_id.to_string(), |plugin| plugin.name.to_string());
        let manifest = PluginPackageManifest {
            plugin_id,
            label,
            source_path: PathBuf::from(self.plugin_package_source.trim()),
            expected_sha256: self.plugin_package_expected_sha256.trim().to_string(),
        };

        match PluginPackageManager::install(&manifest, &node, self.node_work_dir(&node)) {
            Ok(installation) => {
                if let Err(error) = self.repository.upsert_plugin_installation(&installation) {
                    self.notice = Some(error.to_string());
                    return;
                }
                let message = format!(
                    "{} package installed for {} at {} ({} files, {})",
                    plugin_id,
                    node.name,
                    short_path(&installation.installed_path, 54),
                    installation.installed_files,
                    format_bytes(installation.expanded_bytes)
                );
                self.record_node_event(
                    &node,
                    EventKind::PluginInstalled,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
