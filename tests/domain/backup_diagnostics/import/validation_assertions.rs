use neo_nexus::backup::WorkspaceBackup;

use super::*;

pub(super) fn assert_rejects_unsafe_backup_shapes(target: &Repository, backup: &WorkspaceBackup) {
    let mut unsafe_backup = backup.clone();
    unsafe_backup
        .workspace_settings
        .push(WorkspaceSettingBackup {
            key: "unsupported.secret".to_string(),
            value: "do-not-import".to_string(),
        });
    let error = WorkspaceBackupImporter::import(target, &unsafe_backup).unwrap_err();
    assert!(error.to_string().contains("unsupported workspace setting"));
    let validation_error = WorkspaceBackupImporter::validate(&unsafe_backup).unwrap_err();
    assert!(validation_error
        .to_string()
        .contains("unsupported workspace setting"));

    let mut duplicate_node_backup = backup.clone();
    duplicate_node_backup
        .nodes
        .push(duplicate_node_backup.nodes[0].clone());
    let error = WorkspaceBackupImporter::validate(&duplicate_node_backup).unwrap_err();
    assert!(error.to_string().contains("duplicate backup node id"));

    let mut duplicate_port_backup = backup.clone();
    let mut duplicate_port_node = duplicate_port_backup.nodes[0].clone();
    duplicate_port_node.id = "restore-source-port-conflict".to_string();
    duplicate_port_node.name = "restore port conflict".to_string();
    duplicate_port_backup.nodes.push(duplicate_port_node);
    let error = WorkspaceBackupImporter::validate(&duplicate_port_backup).unwrap_err();
    assert!(error.to_string().contains("duplicate backup node port"));

    let mut duplicate_plugin_backup = backup.clone();
    let duplicate_plugin = duplicate_plugin_backup.nodes[0].plugins[0].clone();
    duplicate_plugin_backup.nodes[0]
        .plugins
        .push(duplicate_plugin);
    let error = WorkspaceBackupImporter::validate(&duplicate_plugin_backup).unwrap_err();
    assert!(error.to_string().contains("duplicate plugin state"));

    let mut duplicate_profile_backup = backup.clone();
    duplicate_profile_backup
        .runtime_catalog_profiles
        .push(duplicate_profile_backup.runtime_catalog_profiles[0].clone());
    let error = WorkspaceBackupImporter::validate(&duplicate_profile_backup).unwrap_err();
    assert!(error
        .to_string()
        .contains("duplicate backup runtime catalog profile id"));

    let mut duplicate_remote_backup = backup.clone();
    duplicate_remote_backup
        .remote_servers
        .push(duplicate_remote_backup.remote_servers[0].clone());
    let error = WorkspaceBackupImporter::validate(&duplicate_remote_backup).unwrap_err();
    assert!(error
        .to_string()
        .contains("duplicate backup remote server profile id"));
}
