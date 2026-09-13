//! The node editor: one form serving both add and edit.
//!
//! Every interaction is a plain form post. Changing the client re-renders with
//! that client's storage choices, "Suggest free ports" asks the planner for a
//! block nothing else claims, and a rejected save comes back with the operator's
//! own text still in the boxes and the reason beside the field it belongs to.

mod fields;
mod presets;

pub use fields::*;
pub use presets::role_presets_picker;

use axum::{
    extract::{Form, Path, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    catalog::{PluginCatalog, PluginId},
    core::{
        node::{NewNode, NodeType},
        operations::{EventKind, EventSeverity, NewRuntimeEvent},
    },
    runtime::RuntimeInstallation,
    signing::{SignerBackendProfile, SignerKeyRef},
    types::NodeConfig,
    web::{
        html,
        node_form::{DraftOutcome, FieldErrors, NodeDraft},
        WebState,
    },
};

/// Where the form posts, and what its primary button says.
pub enum EditorMode {
    Create,
    Edit { id: String },
}

impl EditorMode {
    fn post_target(&self) -> String {
        match self {
            Self::Create => "/nodes/new".to_string(),
            Self::Edit { id } => format!("/nodes/{}/edit", html::urlencoding_lite(id)),
        }
    }

    fn cancel_target(&self) -> String {
        match self {
            Self::Create => "/nodes".to_string(),
            Self::Edit { id } => format!("/nodes/{}", html::urlencoding_lite(id)),
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Self::Create => "Add node",
            Self::Edit { .. } => "Edit node",
        }
    }

    fn submit_label(&self) -> &'static str {
        match self {
            Self::Create => "Add node",
            Self::Edit { .. } => "Save changes",
        }
    }

    fn current_id(&self) -> Option<&str> {
        match self {
            Self::Create => None,
            Self::Edit { id } => Some(id),
        }
    }
}

pub async fn new_form(State(state): State<WebState>) -> Response {
    let installations = state.workspace.list_runtime_installations().unwrap_or_default();
    let nodes = load_nodes(&state);
    let profiles = state.custody().profiles().cloned().collect::<Vec<_>>();
    let bindings = state.workspace.list_all_signer_bindings().unwrap_or_default();
    render(
        &NodeDraft::blank_with_installations_and_fleet(&installations, &nodes),
        &EditorMode::Create,
        &FieldErrors::new(),
        &installations,
        &profiles,
        &bindings,
    )
}

pub async fn edit_form(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let installations = state.workspace.list_runtime_installations().unwrap_or_default();
    let profiles = state.custody().profiles().cloned().collect::<Vec<_>>();
    let bindings = state.workspace.list_all_signer_bindings().unwrap_or_default();
    match find_node(&state, &id) {
        Some(node) => {
            let role = state.workspace.load_node_role(&node.id).ok().flatten();
            let plugins = state.workspace.list_plugin_states(&node.id).unwrap_or_default();
            let signer = state.workspace.load_node_signer_key(&node.id).ok().flatten();
            render(
                &NodeDraft::from_node_with_role_plugins_and_signer(
                    &node,
                    role,
                    &plugins,
                    signer.as_ref(),
                ),
                &EditorMode::Edit { id: node.id },
                &FieldErrors::new(),
                &installations,
                &profiles,
                &bindings,
            )
        }
        None => Redirect::to("/nodes").into_response(),
    }
}

pub async fn create(State(state): State<WebState>, Form(form): Form<NodeDraft>) -> Response {
    submit(&state, form, &EditorMode::Create)
}

pub async fn update(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<NodeDraft>,
) -> Response {
    if find_node(&state, &id).is_none() {
        return Redirect::to("/nodes").into_response();
    }
    submit(&state, form, &EditorMode::Edit { id })
}

/// Handle a post. The two non-saving intents are answered before any field is
/// judged, because a half-filled form is not a mistake while the operator is
/// still choosing a client.
fn submit(state: &WebState, form: NodeDraft, mode: &EditorMode) -> Response {
    let installations = state.workspace.list_runtime_installations().unwrap_or_default();
    let profiles = state.custody().profiles().cloned().collect::<Vec<_>>();
    let bindings = state.workspace.list_all_signer_bindings().unwrap_or_default();

    // Read the intent before the draft is normalised, which consumes it.
    let wants_client_defaults = form.wants_client_defaults();
    let wants_suggested_ports = form.wants_suggested_ports();
    let draft = form.with_client_defaults();

    if wants_client_defaults {
        return render(&draft, mode, &FieldErrors::new(), &installations, &profiles, &bindings);
    }
    if wants_suggested_ports {
        let suggested = draft
            .suggest_ports(&load_nodes(state), mode.current_id())
            .unwrap_or_else(|| draft.clone());
        return render(&suggested, mode, &FieldErrors::new(), &installations, &profiles, &bindings);
    }

    // Check IAM isolation for signer key binding
    if let Some(key) = draft.resolved_signer_key() {
        let conflict = bindings.iter().find(|(nid, b)| {
            b == &key && Some(nid.as_str()) != mode.current_id()
        });
        if let Some((owner, _)) = conflict {
            let mut errors = FieldErrors::new();
            errors.insert(
                "signer_backend",
                format!(
                    "IAM Isolation Violation: Signer key '{}/{}' is already exclusively allocated to instance '{owner}'. Cross-node key usage is strictly forbidden.",
                    key.backend_id, key.key_id
                ),
            );
            return render(&draft, mode, &errors, &installations, &profiles, &bindings);
        }
    }

    // `validate` excludes the node being edited by id, so it is given the whole
    // fleet and does the exclusion itself.
    match draft.validate(&load_nodes(state), mode.current_id()) {
        DraftOutcome::Invalid(errors) => {
            render(&draft, mode, &errors, &installations, &profiles, &bindings)
        }
        DraftOutcome::Valid(input) => match mode {
            EditorMode::Create => save_new(state, draft, input, &installations, &profiles, &bindings),
            EditorMode::Edit { id } => save_edit(state, draft, id, input, &installations, &profiles, &bindings),
        },
    }
}

fn save_new(
    state: &WebState,
    draft: NodeDraft,
    input: NewNode,
    installations: &[RuntimeInstallation],
    profiles: &[SignerBackendProfile],
    bindings: &[(String, SignerKeyRef)],
) -> Response {
    let name = input.name.clone();
    match state.commands.create_node(input) {
        Ok(node) => {
            // Persist role
            let role = draft.resolved_role();
            let _ = state.commands.set_node_role(&node.id, role);

            // Persist signer binding if specified
            if let Some(key) = draft.resolved_signer_key() {
                if let Err(e) = state.commands.set_node_signer_key(&node.id, Some(&key)) {
                    log::warn!("Failed to bind signer key to new node {}: {}", node.id, e);
                }
            }

            // Persist plugin states for neo-cli
            if node.node_type == NodeType::NeoCli {
                let selected = draft.selected_plugins();
                for def in PluginCatalog.for_node_type(NodeType::NeoCli) {
                    if !matches!(def.id, PluginId::LevelDbStore | PluginId::RocksDbStore) {
                        let should_enable = selected.contains(&def.id)
                            && (def.id != PluginId::RpcServer || node.rpc_port > 0);
                        let _ = state.commands.set_plugin_enabled(&node.id, def.id, should_enable);
                    }
                }
            }

            // Persist Hermes Agent association if enabled
            if draft.is_hermes_enabled() {
                let assoc = crate::agents::HermesAgentAssociation::new(&node.id);
                let _ = state.commands.save_hermes_agent(&assoc);
            }

            journal(
                state,
                &node.id,
                &node.name,
                EventKind::NodeCreated,
                format!("{name} registered"),
            );
            Redirect::to(&redirect_to(&node.id, &format!("{name} added."))).into_response()
        }
        Err(error) => {
            let mut errors = FieldErrors::new();
            errors.insert("general", error.to_string());
            render(&draft, &EditorMode::Create, &errors, installations, profiles, bindings)
        }
    }
}

fn save_edit(
    state: &WebState,
    draft: NodeDraft,
    id: &str,
    input: NewNode,
    installations: &[RuntimeInstallation],
    profiles: &[SignerBackendProfile],
    bindings: &[(String, SignerKeyRef)],
) -> Response {
    let name = input.name.clone();
    match state.commands.update_node(id, input) {
        Ok(node) => {
            let role = draft.resolved_role();
            let _ = state.commands.set_node_role(&node.id, role);

            // Persist or clear signer binding
            if let Some(key) = draft.resolved_signer_key() {
                let _ = state.commands.set_node_signer_key(id, Some(&key));
            } else if draft.signer_backend.trim().is_empty() && draft.signer_key.trim().is_empty() {
                let _ = state.commands.set_node_signer_key(id, None);
            }

            if node.node_type == NodeType::NeoCli {
                let selected = draft.selected_plugins();
                for def in PluginCatalog.for_node_type(NodeType::NeoCli) {
                    if !matches!(def.id, PluginId::LevelDbStore | PluginId::RocksDbStore) {
                        let should_enable = selected.contains(&def.id)
                            && (def.id != PluginId::RpcServer || node.rpc_port > 0);
                        let _ = state.commands.set_plugin_enabled(&node.id, def.id, should_enable);
                    }
                }
            }

            journal(
                state,
                &node.id,
                &node.name,
                EventKind::NodeUpdated,
                format!("{name} configuration updated"),
            );
            Redirect::to(&redirect_to(&node.id, &format!("{name} updated."))).into_response()
        }
        Err(error) => {
            let mut errors = FieldErrors::new();
            errors.insert("general", error.to_string());
            render(&draft, &EditorMode::Edit { id: id.to_string() }, &errors, installations, profiles, bindings)
        }
    }
}

fn render(
    draft: &NodeDraft,
    mode: &EditorMode,
    errors: &FieldErrors,
    installations: &[RuntimeInstallation],
    profiles: &[SignerBackendProfile],
    bindings: &[(String, SignerKeyRef)],
) -> Response {
    let body = format!(
        r#"{breadcrumb}
{head}
{summary}
<form method="post" action="{target}">
<div class="aws-wizard-container" style="display: flex; flex-direction: column; gap: 16px;">
    <!-- Step 1: Name and tags -->
    <div class="panel aws-wizard-card" style="padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; border-bottom: 1px solid var(--line); padding-bottom: 8px;">
            <div>
                <strong style="font-size: 15px;">1. Name and tags</strong>
                <div class="muted" style="font-size: 12px;">Assign instance identification and architectural duty presets.</div>
            </div>
            <span class="badge">Required</span>
        </div>
        {role_presets}
        <div class="grid">{name}</div>
    </div>

    <!-- Step 2: Application and OS Images (Client Engine / AMI) -->
    <div class="panel aws-wizard-card" style="padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; border-bottom: 1px solid var(--line); padding-bottom: 8px;">
            <div>
                <strong style="font-size: 15px;">2. Application and OS Images (Client Engine &amp; Runtime AMI)</strong>
                <div class="muted" style="font-size: 12px;">Select blockchain client engine, executable binary path, and runtime version.</div>
            </div>
            <span class="badge">AMI Specification</span>
        </div>
        <div class="grid">{client}{binary}{version}{args}</div>
    </div>

    <!-- Step 3: Network settings & Security Groups (Firewall) -->
    <div class="panel aws-wizard-card" style="padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; border-bottom: 1px solid var(--line); padding-bottom: 8px;">
            <div>
                <strong style="font-size: 15px;">3. Network settings &amp; Security Groups (Firewall Rules)</strong>
                <div class="muted" style="font-size: 12px;">Configure network mesh cluster, P2P listener port, and JSON-RPC firewall rules.</div>
            </div>
            <span class="badge">Security Group</span>
        </div>
        <div class="grid">
            {network}
            {p2p}
        </div>
        <div style="margin-top: 12px;">
            {rpc_section}
        </div>
    </div>

    <!-- Step 4: Configure storage (EBS Volume) -->
    <div class="panel aws-wizard-card" style="padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; border-bottom: 1px solid var(--line); padding-bottom: 8px;">
            <div>
                <strong style="font-size: 15px;">4. Configure storage (Block Device / Volume Driver)</strong>
                <div class="muted" style="font-size: 12px;">Select local persistent RocksDB block store or memory storage engine.</div>
            </div>
            <span class="badge">EBS Volume</span>
        </div>
        <div class="grid">{storage}</div>
    </div>

    <!-- Step 5: Security & IAM Identity -->
    <div class="panel aws-wizard-card" style="padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; border-bottom: 1px solid var(--line); padding-bottom: 8px;">
            <div>
                <strong style="font-size: 15px;">5. Security &amp; IAM Identity (Cryptographic Signer Lease)</strong>
                <div class="muted" style="font-size: 12px;">Bind isolated cryptographic consensus signing keys with double-signing prevention.</div>
            </div>
            <span class="badge">IAM Role</span>
        </div>
        {signer_section}
    </div>

    <!-- Step 6: Systems Manager & AI Copilot -->
    <div class="panel aws-wizard-card" style="padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; border-bottom: 1px solid var(--line); padding-bottom: 8px;">
            <div>
                <strong style="font-size: 15px;">6. Systems Manager &amp; AI Copilot (Autonomous Supervision)</strong>
                <div class="muted" style="font-size: 12px;">Enable autonomous self-healing, crash recovery, and guest copilot agent.</div>
            </div>
            <span class="badge">Guest Agent</span>
        </div>
        {hermes_section}
    </div>

    <!-- Step 7: Plugins & Ecosystem Sidecars -->
    <div class="panel aws-wizard-card" style="padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; border-bottom: 1px solid var(--line); padding-bottom: 8px;">
            <div>
                <strong style="font-size: 15px;">7. Modular Plugins &amp; Ecosystem Sidecars</strong>
                <div class="muted" style="font-size: 12px;">Select and configure native plugins for state indexing, application logs, and RPC servers.</div>
            </div>
            <span class="badge">Sidecars</span>
        </div>
        {plugins_section}
    </div>

    <!-- Sticky Launch Summary Bar -->
    <div class="panel aws-wizard-summary" style="padding: 14px 18px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px; position: sticky; bottom: 12px; z-index: 10; box-shadow: 0 4px 20px rgba(0,0,0,0.4);">
        <div style="display: flex; align-items: center; gap: 12px; flex-wrap: wrap;">
            <button class="primary" type="submit" style="padding: 8px 18px; font-weight: 600;">{save}</button>
            <button type="submit" name="suggest" value="1" class="btn small">Suggest free ports</button>
            <button type="submit" name="client" value="1" class="btn small">Apply client defaults</button>
        </div>
        <div>
            <a class="btn small" href="{cancel}">Cancel</a>
        </div>
    </div>
</div>
</form>"#,
        breadcrumb = html::breadcrumb(&[("EC2", "/nodes"), ("Instances", "/nodes"), (mode.title(), "")]),
        head = html::page_head(
            mode.title(),
            "Provision a blockchain virtual machine instance with automated networking, isolated IAM identity, and guest agent copilot.",
            "",
        ),
        summary = summary(errors),
        target = mode.post_target(),
        cancel = mode.cancel_target(),
        save = mode.submit_label(),
        role_presets = role_presets_picker(draft),
        name = name_field(draft, errors),
        client = client_field(draft, errors),
        network = network_field(draft, errors),
        storage = storage_field(draft, errors),
        binary = binary_field(draft, errors, installations),
        version = version_field(draft, errors),
        args = args_field(draft, errors),
        p2p = p2p_field(draft, errors),
        rpc_section = rpc_section(draft, errors),
        plugins_section = plugins_section(draft),
        hermes_section = hermes_section(draft),
        signer_section = signer_section(draft, errors, profiles, bindings, mode.current_id()),
    );
    Html(html::layout("Node", "nodes", "", &body)).into_response()
}

fn summary(errors: &FieldErrors) -> String {
    let general = errors
        .get("general")
        .map(|message| html::notice("danger", message))
        .unwrap_or_default();
    let count = errors
        .len()
        .saturating_sub(usize::from(errors.contains_key("general")));
    let header = match count {
        0 => String::new(),
        1 => html::notice("danger", "One field needs attention."),
        count => html::notice("danger", &format!("{count} fields need attention.")),
    };
    format!("{header}{general}")
}

fn load_nodes(state: &WebState) -> Vec<NodeConfig> {
    state.workspace.list_nodes().unwrap_or_default()
}

fn find_node(state: &WebState, id: &str) -> Option<NodeConfig> {
    load_nodes(state).into_iter().find(|node| node.id == id)
}

fn redirect_to(id: &str, message: &str) -> String {
    format!(
        "/nodes/{}?flash={}",
        html::urlencoding_lite(id),
        html::urlencoding_lite(message)
    )
}

/// The journal is an audit trail, not a precondition: a failed write must not
/// make a completed registration look like it never happened.
fn journal(state: &WebState, node_id: &str, node_name: &str, kind: EventKind, message: String) {
    let _ = state.commands.record_event(NewRuntimeEvent {
        node_id: Some(node_id.to_string()),
        node_name: Some(node_name.to_string()),
        kind,
        severity: EventSeverity::Info,
        message,
    });
}
