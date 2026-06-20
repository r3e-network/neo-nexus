use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn config_export_dir(&self) -> PathBuf {
        self.workspace_child_dir("configs")
    }

    pub(in crate::app) fn config_export_path(&self, node: &NodeConfig) -> PathBuf {
        ConfigExporter::target_path(self.config_export_dir(), node)
    }

    pub(in crate::app) fn managed_config_path(&self, node: &NodeConfig) -> PathBuf {
        ConfigExporter::managed_target_path(self.node_work_dir(node), node)
    }

    pub(in crate::app) fn log_dir(&self) -> PathBuf {
        self.workspace_child_dir("logs")
    }

    pub(in crate::app) fn node_log_path(&self, node: &NodeConfig) -> PathBuf {
        log_path_for(self.log_dir(), node)
    }

    pub(in crate::app) fn launch_plan_for(&self, node: &NodeConfig) -> LaunchPlan {
        LaunchPlanner::plan(
            node,
            self.managed_config_path(node),
            self.node_work_dir(node),
        )
    }

    pub(in crate::app) fn node_root_dir(&self) -> PathBuf {
        self.workspace_child_dir("nodes")
    }

    pub(in crate::app) fn node_work_dir(&self, node: &NodeConfig) -> PathBuf {
        self.node_root_dir().join(&node.id)
    }

    pub(in crate::app) fn node_data_dir(&self, node: &NodeConfig) -> PathBuf {
        self.node_work_dir(node)
            .join("data")
            .join(node.network.to_string())
    }

    pub(in crate::app) fn backup_export_dir(&self) -> PathBuf {
        self.workspace_child_dir("backups")
    }

    pub(in crate::app) fn readiness_report_dir(&self) -> PathBuf {
        self.workspace_child_dir("reports")
    }

    pub(in crate::app) fn event_journal_export_dir(&self) -> PathBuf {
        self.workspace_child_dir("events")
    }

    pub(in crate::app) fn support_bundle_dir(&self) -> PathBuf {
        self.workspace_child_dir("support")
    }

    pub(in crate::app) fn release_package_dir(&self) -> PathBuf {
        self.workspace_child_dir("dist")
    }

    pub(in crate::app) fn snapshot_cache_dir(&self) -> PathBuf {
        self.workspace_child_dir("snapshots")
    }

    pub(in crate::app) fn runtime_install_root(&self) -> PathBuf {
        self.workspace_child_dir("runtimes")
    }

    pub(in crate::app) fn runtime_download_dir(&self) -> PathBuf {
        self.workspace_child_dir("runtime-downloads")
    }

    pub(in crate::app) fn private_network_export_dir(&self) -> PathBuf {
        self.workspace_child_dir("private-networks")
    }

    pub(in crate::app) fn runtime_installations(&self) -> Vec<RuntimeInstallation> {
        self.repository
            .list_runtime_installations()
            .unwrap_or_default()
    }

    fn workspace_child_dir(&self, fallback: &str) -> PathBuf {
        self.repository
            .db_path()
            .parent()
            .map_or_else(|| PathBuf::from(fallback), |parent| parent.join(fallback))
    }
}
