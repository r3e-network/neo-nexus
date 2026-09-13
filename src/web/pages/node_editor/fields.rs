//! Form field renderers for the node creation & edit workbench.

use crate::{
    catalog::{PluginCatalog, PluginId},
    core::node::{Network, NodeType, StorageEngine},
    runtime::RuntimeInstallation,
    signing::SignerBackendProfile,
    web::{
        html,
        node_form::{FieldErrors, NodeDraft},
    },
};

pub fn error_for<'a>(errors: &'a FieldErrors, key: &str) -> Option<&'a str> {
    errors.get(key).map(String::as_str)
}

pub fn labels<T: std::fmt::Display, const N: usize>(values: [T; N]) -> Vec<String> {
    values.iter().map(T::to_string).collect()
}

pub fn name_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    html::TextField {
        label: "Node name",
        name: "name",
        value: &draft.name,
        error: error_for(errors, "name"),
        help: Some("Shows in the fleet list and names the node's log and config files."),
        full_width: true,
        ..html::TextField::default()
    }
    .render()
}

pub fn client_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    let options = labels(NodeType::ALL);
    html::ChoiceField {
        label: "Client",
        name: "node_type",
        options: &options,
        selected: &draft.node_type,
        error: error_for(errors, "node_type"),
        help: Some("Decides which storage engines and plugins apply."),
        auto_submit: Some("client"),
        ..html::ChoiceField::default()
    }
    .render()
}

pub fn network_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    let options = labels(Network::ALL);
    html::ChoiceField {
        label: "Network",
        name: "network",
        options: &options,
        selected: &draft.network,
        error: error_for(errors, "network"),
        ..html::ChoiceField::default()
    }
    .render()
}

pub fn storage_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    let error = error_for(errors, "storage_engine");
    if !draft.storage_is_selectable() {
        let note = draft.storage_note().unwrap_or_else(|| {
            "Choose a client to see the storage engines it supports.".to_string()
        });
        let marked = error
            .map(|message| html::notice("danger", message))
            .unwrap_or_default();
        return format!(
            r#"<div class="field span-all"><span>Storage engine</span>{marked}<span class="help">{note}</span></div>"#,
            marked = marked,
            note = html::escape(&note),
        );
    }
    let options = labels(StorageEngine::ALL);
    html::ChoiceField {
        label: "Storage engine",
        name: "storage_engine",
        options: &options,
        selected: &draft.storage_engine,
        error,
        help: Some("Only engines the selected client supports can be saved."),
        ..html::ChoiceField::default()
    }
    .render()
}

pub fn binary_field(
    draft: &NodeDraft,
    errors: &FieldErrors,
    installations: &[RuntimeInstallation],
) -> String {
    let client_type = draft.parsed_type();
    let matching_installations: Vec<&RuntimeInstallation> = installations
        .iter()
        .filter(|inst| client_type.is_none_or(|ct| inst.node_type == ct))
        .collect();

    let text_input = html::TextField {
        id: Some("node_binary_path"),
        label: "Node binary (executable path or command)",
        name: "binary_path",
        value: &draft.binary_path,
        error: error_for(errors, "binary_path"),
        help: Some(
            "Auto-infers client type from path/filename, or select from installed runtimes below.",
        ),
        monospace: true,
        full_width: true,
        placeholder: Some("/opt/neo/neo-go or C:\\runtimes\\neo-node.exe"),
    }
    .render();

    let inference_badge = r#"<div id="binary-inference-badge" class="binary-inference-badge" style="display:none;"></div>"#;

    let picker_html = if !matching_installations.is_empty() {
        let chips = matching_installations
            .iter()
            .map(|inst| {
                let verified_badge = if inst.signature_verified { " 🛡️ verified" } else { "" };
                let is_current = draft.binary_path.trim() == inst.binary_path.display().to_string();
                let active_class = if is_current { " runtime-chip-active" } else { "" };
                format!(
                    r#"<button type="button" class="runtime-chip{active}" data-binary="{path}" data-version="{version}" data-client="{client}" title="Use this installed runtime binary">
                        <span class="chip-icon">⚡</span>
                        <span class="chip-title"><strong>{label}</strong> v{version}{verified}</span>
                    </button>"#,
                    active = active_class,
                    path = html::escape(&inst.binary_path.display().to_string()),
                    version = html::escape(&inst.version),
                    client = html::escape(&inst.node_type.to_string()),
                    label = html::escape(&inst.label),
                    verified = verified_badge,
                )
            })
            .collect::<Vec<_>>()
            .join("");

        let client_label = client_type.map_or_else(|| "all clients".to_string(), |c| c.to_string());
        format!(
            r#"<div class="runtime-picker-box">
                <div class="runtime-picker-header">
                    <span class="picker-lead">📦 Discovered Runtime Installations ({client_label}):</span>
                    <span class="picker-help">Click to auto-populate binary path and runtime version</span>
                </div>
                <div class="runtime-chips-grid">
                    {chips}
                </div>
            </div>"#
        )
    } else {
        String::new()
    };

    format!("{text_input}\n{inference_badge}\n{picker_html}")
}

pub fn version_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    html::TextField {
        id: Some("node_runtime_version"),
        label: "Runtime version",
        name: "runtime_version",
        value: &draft.runtime_version,
        error: error_for(errors, "runtime_version"),
        help: Some("Leave blank to match the latest installed runtime."),
        placeholder: Some("latest"),
        ..html::TextField::default()
    }
    .render()
}

pub fn args_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    html::TextField {
        label: "Extra arguments",
        name: "args",
        value: &draft.args,
        error: error_for(errors, "args"),
        help: Some("Quoted values keep their spaces. Secrets belong in the node config, not here."),
        monospace: true,
        full_width: true,
        ..html::TextField::default()
    }
    .render()
}

pub fn p2p_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    html::TextField {
        label: "P2P port",
        name: "p2p_port",
        value: &draft.p2p_port,
        error: error_for(errors, "p2p_port"),
        ..html::TextField::default()
    }
    .render()
}

pub fn rpc_section(draft: &NodeDraft, errors: &FieldErrors) -> String {
    let is_rpc_on = draft.is_rpc_enabled();
    let checked_attr = if is_rpc_on { "checked" } else { "" };
    let panel_style = if is_rpc_on { "" } else { "display: none;" };
    let notice_style = if is_rpc_on { "display: none;" } else { "" };
    let status_badge = if is_rpc_on {
        r#"<span class="badge running">RPC Enabled</span>"#
    } else {
        r#"<span class="badge stopped">RPC Disabled (P2P Only)</span>"#
    };

    let rpc_port_input = html::TextField {
        label: "RPC port",
        name: "rpc_port",
        value: &draft.rpc_port,
        error: error_for(errors, "rpc_port"),
        help: Some("Main JSON-RPC HTTP port (e.g. 10332)."),
        ..html::TextField::default()
    }
    .render();

    let ws_port_input = html::TextField {
        label: "WebSocket port",
        name: "ws_port",
        value: &draft.ws_port,
        error: error_for(errors, "ws_port"),
        help: Some("Optional WebSocket port for block/event subscriptions."),
        ..html::TextField::default()
    }
    .render();

    format!(
        r#"<div class="field span-all rpc-capability-box">
            <input type="hidden" name="rpc_configured" value="1">
            <div class="rpc-toggle-header">
                <label class="toggle-control">
                    <input type="checkbox" id="f-enable_rpc" name="enable_rpc" value="1" {checked_attr} onchange="toggleNodeRpcService(this.checked)">
                    <span class="toggle-slider"></span>
                    <span class="toggle-label-text"><strong>Enable JSON-RPC 2.0 API Service</strong> (Expose endpoints for wallets, dApps & explorer probes)</span>
                </label>
                <div id="rpc-status-tag">{status_badge}</div>
            </div>
            <div id="rpc-config-panel" class="rpc-config-body" style="{panel_style}">
                <div class="grid rpc-ports-grid">
                    {rpc_port_input}
                    {ws_port_input}
                </div>
            </div>
            <div id="rpc-disabled-notice" class="rpc-disabled-banner" style="{notice_style}">
                <span class="notice-icon">🔒</span>
                <div class="notice-body">
                    <strong>RPC API service is disabled.</strong>
                    <div>This node will operate strictly on the peer-to-peer gossip protocol with zero open RPC attack surface. Port 0 is recorded in inventory and no local RPC ports are opened.</div>
                </div>
            </div>
        </div>"#,
        checked_attr = checked_attr,
        status_badge = status_badge,
        panel_style = panel_style,
        notice_style = notice_style,
        rpc_port_input = rpc_port_input,
        ws_port_input = ws_port_input,
    )
}

pub fn plugins_section(draft: &NodeDraft) -> String {
    let node_type = draft.parsed_type().unwrap_or(NodeType::NeoCli);
    if node_type != NodeType::NeoCli {
        let guidance = match node_type {
            NodeType::NeoGo => {
                "NeoGo manages services (RPC, Prometheus, pprof) directly in configuration without DLL plugins."
            }
            NodeType::NeoRs => "neo-rs configures RPC and consensus directly via TOML.",
            NodeType::NeoXGeth => {
                "Neo X Geth configures RPC modules and peering directly in TOML."
            }
            NodeType::NeoXReth => "Neo X Reth uses launch flags and built-in Reth extensions.",
            NodeType::NeoCli => "neo-cli manages plugins via verified DLL packages.",
        };
        return format!(
            r#"<div class="field span-all runtime-plugins-box">
                <div class="section-lead-label">
                    <span class="lead-icon">🧩</span>
                    <div>
                        <strong>Services & Extensions ({node_type})</strong>
                        <div class="help">{guidance}</div>
                    </div>
                </div>
            </div>"#
        );
    }

    let catalog = PluginCatalog;
    let available_plugins = catalog.for_node_type(NodeType::NeoCli);
    let selected_plugins = draft.selected_plugins();

    let items = available_plugins
        .iter()
        .filter(|p| !matches!(p.id, PluginId::LevelDbStore | PluginId::RocksDbStore))
        .map(|plugin| {
            let is_checked = selected_plugins.contains(&plugin.id)
                || (draft.plugins.is_empty() && plugin.id == PluginId::RpcServer);
            let checked_attr = if is_checked { "checked" } else { "" };
            let cat_label = match plugin.category {
                crate::catalog::PluginCategory::Api => "API",
                crate::catalog::PluginCategory::Indexing => "INDEXER",
                crate::catalog::PluginCategory::Governance => "GOVERNANCE",
                crate::catalog::PluginCategory::Core => "CORE",
                crate::catalog::PluginCategory::Storage => "STORAGE",
            };
            format!(
                r#"<label class="plugin-selection-card" data-plugin-id="{id}">
                    <input type="checkbox" name="plugins" value="{id}" class="plugin-checkbox" data-plugin="{id}" {checked_attr}>
                    <div class="plugin-card-content">
                        <div class="plugin-card-header">
                            <span class="plugin-name">{name}</span>
                            <span class="badge">{cat_label}</span>
                        </div>
                        <div class="plugin-desc">{desc}</div>
                    </div>
                </label>"#,
                id = plugin.id,
                name = html::escape(plugin.name),
                cat_label = cat_label,
                desc = html::escape(plugin.description),
                checked_attr = checked_attr,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<div class="field span-all plugins-selection-section">
            <div class="section-lead-label">
                <span class="lead-icon">🧩</span>
                <div>
                    <strong>Plugins & Extensions (neo-cli)</strong>
                    <div class="help">Select which verified plugin assemblies to enable for this node. Role presets automatically configure recommended plugins.</div>
                </div>
            </div>
            <div class="plugins-selection-grid">
                {items}
            </div>
        </div>"#,
        items = items,
    )
}

pub fn hermes_section(draft: &NodeDraft) -> String {
    let checked = if draft.is_hermes_enabled() {
        "checked"
    } else {
        ""
    };
    format!(
        r#"<div class="field span-all hermes-provision-box" style="margin-top: 14px; padding-top: 14px; border-top: 1px solid var(--line);">
            <div class="section-lead-label">
                <span class="lead-icon">🤖</span>
                <div>
                    <strong>Hermes AI Guest Agent (Autonomous Copilot)</strong>
                    <div class="help">Associate an autonomous guest agent to monitor telemetry, tail logs, and execute self-healing via scoped Model Context Protocol (MCP).</div>
                </div>
            </div>
            <label class="toggle-control" style="margin-top: 10px;">
                <input type="checkbox" name="hermes_enabled" value="1" {checked}>
                <span class="toggle-slider"></span>
                <span class="toggle-label-text"><strong>Enable Hermes Agent Integration</strong> (Scoped MCP tool execution, health heartbeat, and autonomous recovery)</span>
            </label>
        </div>"#,
        checked = checked,
    )
}

pub fn signer_section(
    draft: &NodeDraft,
    errors: &FieldErrors,
    profiles: &[SignerBackendProfile],
    existing_bindings: &[(String, crate::signing::SignerKeyRef)],
    current_node_id: Option<&str>,
) -> String {
    let error = error_for(errors, "signer_backend").or_else(|| error_for(errors, "signer_key"));
    let marked = error
        .map(|message| html::notice("danger", message))
        .unwrap_or_default();

    let mut options =
        vec![r#"<option value="">No signer (read-only duty / unleased)</option>"#.to_string()];
    for profile in profiles {
        let is_selected = draft.signer_backend == profile.id;
        let other_owner = existing_bindings
            .iter()
            .find(|(nid, b)| b.backend_id == profile.id && Some(nid.as_str()) != current_node_id);
        if let Some((owner_id, _)) = other_owner {
            options.push(format!(
                r#"<option value="{id}" disabled>🔒 {label} ({kind}) — Leased to {owner_id}</option>"#,
                id = html::escape(&profile.id),
                label = html::escape(&profile.label),
                kind = html::escape(profile.kind.label()),
                owner_id = html::escape(owner_id),
            ));
        } else {
            let chosen = if is_selected { " selected" } else { "" };
            options.push(format!(
                r#"<option value="{id}"{chosen}>🔑 {label} ({kind})</option>"#,
                id = html::escape(&profile.id),
                label = html::escape(&profile.label),
                kind = html::escape(profile.kind.label()),
            ));
        }
    }
    let options_html = options.join("\n");

    let signer_key_input = html::TextField {
        id: Some("node-signer-key"),
        label: "Key identifier / Public key",
        name: "signer_key",
        value: &draft.signer_key,
        error: error_for(errors, "signer_key"),
        help: Some("The key ID or public key managed by this signer backend."),
        monospace: true,
        placeholder: Some("e.g. validator-key or 02..."),
        ..html::TextField::default()
    }
    .render();

    format!(
        r#"<div class="field span-all signer-capability-box" style="margin-top: 14px; padding-top: 14px; border-top: 1px solid var(--line);">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span style="font-size: 18px;">🔒</span>
                    <strong>IAM Instance Profile & Signer Identity</strong>
                </div>
                <span class="badge" style="background: rgba(46, 194, 116, 0.15); color: #2ec274;">Cryptographic Lease Isolation</span>
            </div>
            {marked}
            <p class="muted" style="font-size: 13px; margin: 4px 0 10px 0;">
                Lease an exclusive signer key or wallet to this virtual instance. Consensus Validators require an active signer lease for block signing. Cross-node key sharing is strictly prohibited.
            </p>
            <div class="grid" style="grid-template-columns: 1fr 1fr; gap: 10px;">
                <label class="field" for="node-signer-backend">
                    <span>Signer Backend Profile</span>
                    <select id="node-signer-backend" name="signer_backend">
                        {options_html}
                    </select>
                    <span class="help">Available signer profiles from custody registry. Locked profiles are leased to other instances.</span>
                </label>
                {signer_key_input}
            </div>
        </div>"#,
        marked = marked,
        options_html = options_html,
        signer_key_input = signer_key_input,
    )
}
