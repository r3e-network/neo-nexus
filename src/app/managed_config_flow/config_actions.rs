mod apply;
mod export;
mod restart;

use crate::app::domain::{ConfigExport, PluginState};

use super::*;

fn selected_node_or_notice(app: &mut NeoNexusApp, notice: &str) -> Option<NodeConfig> {
    let Some(node) = app.selected_node().cloned() else {
        app.session.notice = Some(notice.to_string());
        return None;
    };
    Some(node)
}

fn plugin_states_for(app: &NeoNexusApp, node: &NodeConfig) -> Vec<PluginState> {
    app.repository
        .list_plugin_states(&node.id)
        .unwrap_or_default()
}

fn node_requires_restart(node: &NodeConfig) -> bool {
    node.status.is_active()
}

fn record_managed_config_applied(
    app: &mut NeoNexusApp,
    node: &NodeConfig,
    export: &ConfigExport,
    running: bool,
) {
    let message = managed_config_applied_message(node, export, running);
    app.record_node_event(
        node,
        EventKind::ConfigApplied,
        if running {
            EventSeverity::Warning
        } else {
            EventSeverity::Info
        },
        message.clone(),
    );
    app.session.notice = Some(message);
}

fn managed_config_applied_message(
    node: &NodeConfig,
    export: &ConfigExport,
    running: bool,
) -> String {
    if running {
        format!(
            "Managed config staged for {}; restart or runtime reload required: {}",
            node.name,
            short_path(&export.path, 54)
        )
    } else {
        format!(
            "Managed config applied for {}: {} ({} bytes)",
            node.name,
            short_path(&export.path, 54),
            export.bytes_written
        )
    }
}

/// The generation context for a node: the duty it is recorded as performing.
///
/// A wallet is deliberately absent. Enabling a signing service writes a wallet
/// password to the config file in plaintext, so it is only ever supplied by an
/// operator through an explicit export, never inferred here.
pub(in crate::app) fn generation_context_for(
    app: &NeoNexusApp,
    node: &NodeConfig,
) -> GenerationContext {
    GenerationContext {
        role: app.repository.load_node_role(&node.id).unwrap_or_default(),
        wallet: None,
    }
}
