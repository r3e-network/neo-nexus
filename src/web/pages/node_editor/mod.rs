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

use std::collections::BTreeMap;

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

/// What the editor needs to render itself, read once per request.
///
/// These four were threaded through `render`, `submit`, `save_new` and
/// `save_edit` as separate parameters, and every new one the form needed —
/// the fleet, so a lease can be reported by instance name rather than by
/// uuid — widened five signatures at once.
pub struct EditorContext {
    pub(super) installations: Vec<RuntimeInstallation>,
    pub(super) profiles: Vec<SignerBackendProfile>,
    /// Every signer lease in the workspace, as `(node id, key)`.
    pub(super) leases: Vec<(String, SignerKeyRef)>,
    pub(super) fleet: Vec<NodeConfig>,
    /// Backend id → the one key it owns, where the workspace knows it without
    /// asking a custody service.
    pub(super) sole_keys: BTreeMap<String, String>,
}

impl EditorContext {
    fn load(state: &WebState) -> Self {
        Self {
            installations: state
                .workspace
                .list_runtime_installations()
                .unwrap_or_default(),
            profiles: state.custody().profiles().cloned().collect(),
            leases: state
                .workspace
                .list_all_signer_bindings()
                .unwrap_or_default(),
            fleet: state.workspace.list_nodes().unwrap_or_default(),
            sole_keys: state.custody().registry().sole_key_ids(),
        }
    }

    /// Name an instance the way the operator does.
    pub(super) fn name_of(&self, node_id: &str) -> String {
        crate::web::fleet::instance_namer(&self.fleet)(node_id)
    }

    /// Fill in the key a chosen backend owns, when the operator left it blank
    /// and there is only one it could be.
    ///
    /// A local wallet and a local signer each hold exactly one key, and the
    /// save path already compared what was typed against it and refused a
    /// mismatch. Asking for a value we hold, in order to reject it, is work the
    /// operator should never have been given.
    fn with_resolved_signer_key(&self, mut draft: NodeDraft) -> NodeDraft {
        let backend = draft.signer_backend.trim();
        if backend.is_empty() || !draft.signer_key.trim().is_empty() {
            return draft;
        }
        if let Some(key_id) = self.sole_keys.get(backend) {
            draft.signer_key = key_id.clone();
        }
        draft
    }
}

pub async fn new_form(State(state): State<WebState>) -> Response {
    let context = EditorContext::load(&state);
    render(
        &NodeDraft::blank_with_installations_and_fleet(&context.installations, &context.fleet),
        &EditorMode::Create,
        &FieldErrors::new(),
        &context,
    )
}

pub async fn edit_form(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let context = EditorContext::load(&state);
    match context.fleet.iter().find(|node| node.id == id) {
        Some(node) => {
            let role = state.workspace.load_node_role(&node.id).ok().flatten();
            let plugins = state
                .workspace
                .list_plugin_states(&node.id)
                .unwrap_or_default();
            let signer = state
                .workspace
                .load_node_signer_key(&node.id)
                .ok()
                .flatten();
            let draft = NodeDraft::from_node_with_role_plugins_and_signer(
                node,
                role,
                &plugins,
                signer.as_ref(),
            );
            render(
                &draft,
                &EditorMode::Edit {
                    id: node.id.clone(),
                },
                &FieldErrors::new(),
                &context,
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
    let context = EditorContext::load(state);

    // Read the intent before the draft is normalised, which consumes it.
    let wants_client_defaults = form.wants_client_defaults();
    let wants_suggested_ports = form.wants_suggested_ports();
    let draft = context.with_resolved_signer_key(form.with_client_defaults());

    if wants_client_defaults {
        return render(&draft, mode, &FieldErrors::new(), &context);
    }
    if wants_suggested_ports {
        let suggested = draft
            .suggest_ports(&context.fleet, mode.current_id())
            .unwrap_or_else(|| draft.clone());
        return render(&suggested, mode, &FieldErrors::new(), &context);
    }

    // One key, one instance — the same rule and the same wording the detail page
    // and the repository write use, rather than a third phrasing of it.
    if let Some(key) = draft.resolved_signer_key() {
        if let Err(violation) = crate::signing::check_signer_binding_allowed(
            mode.current_id().unwrap_or_default(),
            &key,
            &context.leases,
            |node_id| context.name_of(node_id),
        ) {
            let mut errors = FieldErrors::new();
            errors.insert("signer_backend", violation.to_string());
            return render(&draft, mode, &errors, &context);
        }
    }

    // `validate` excludes the node being edited by id, so it is given the whole
    // fleet and does the exclusion itself.
    match draft.validate(&context.fleet, mode.current_id()) {
        DraftOutcome::Invalid(errors) => render(&draft, mode, &errors, &context),
        DraftOutcome::Valid(input) => match mode {
            EditorMode::Create => save_new(state, draft, input, &context),
            EditorMode::Edit { id } => save_edit(state, draft, id, input, &context),
        },
    }
}

fn save_new(
    state: &WebState,
    draft: NodeDraft,
    input: NewNode,
    context: &EditorContext,
) -> Response {
    let name = input.name.clone();
    match state.commands.create_node(input) {
        Ok(node) => {
            let mut unapplied = apply_node_settings(state, &draft, &node);

            // Persist Hermes Agent association if enabled.
            if draft.is_hermes_enabled() {
                let assoc = crate::agents::HermesAgentAssociation::new(&node.id);
                if let Err(error) = state.commands.save_hermes_agent(&assoc) {
                    unapplied.push(format!("guest agent not enabled: {error}"));
                }
            }

            journal(
                state,
                &node.id,
                &node.name,
                EventKind::NodeCreated,
                format!("{name} registered"),
            );
            Redirect::to(&redirect_to(
                &node.id,
                &outcome_message(&format!("{name} added."), &unapplied),
            ))
            .into_response()
        }
        Err(error) => {
            let mut errors = FieldErrors::new();
            errors.insert("general", error.to_string());
            render(&draft, &EditorMode::Create, &errors, context)
        }
    }
}

fn save_edit(
    state: &WebState,
    draft: NodeDraft,
    id: &str,
    input: NewNode,
    context: &EditorContext,
) -> Response {
    let name = input.name.clone();
    match state.commands.update_node(id, input) {
        Ok(node) => {
            let unapplied = apply_node_settings(state, &draft, &node);
            journal(
                state,
                &node.id,
                &node.name,
                EventKind::NodeUpdated,
                format!("{name} configuration updated"),
            );
            Redirect::to(&redirect_to(
                &node.id,
                &outcome_message(&format!("{name} updated."), &unapplied),
            ))
            .into_response()
        }
        Err(error) => {
            let mut errors = FieldErrors::new();
            errors.insert("general", error.to_string());
            render(
                &draft,
                &EditorMode::Edit { id: id.to_string() },
                &errors,
                context,
            )
        }
    }
}

/// Apply the settings that live beside the node row — duty, signer lease and
/// plugin state — and return whatever could not be applied.
///
/// The node itself is already saved by the time these run, so a failure here
/// cannot be reported by re-rendering the form. It used to be swallowed
/// entirely: a signer lease the workspace refused was logged at warn level and
/// the operator was told "added", leaving a node whose recorded identity was
/// not the one they had just filled in. Naming the failures in the flash is the
/// least an operator needs to know the form did not fully take.
fn apply_node_settings(state: &WebState, draft: &NodeDraft, node: &NodeConfig) -> Vec<String> {
    let mut unapplied = Vec::new();

    if let Err(error) = state
        .commands
        .set_node_role(&node.id, draft.resolved_role())
    {
        unapplied.push(format!("duty not applied: {error}"));
    }

    match draft.resolved_signer_key() {
        Some(key) => {
            if let Err(error) = state.commands.set_node_signer_key(&node.id, Some(&key)) {
                unapplied.push(format!("signer lease not applied: {error}"));
            }
        }
        // A blank pair is an instruction to release the lease, not an absence
        // of intent. A half-filled one was already rejected by validation.
        None if draft.signer_backend.trim().is_empty() && draft.signer_key.trim().is_empty() => {
            if let Err(error) = state.commands.set_node_signer_key(&node.id, None) {
                unapplied.push(format!("signer lease not cleared: {error}"));
            }
        }
        None => {}
    }

    if node.node_type == NodeType::NeoCli {
        let selected = draft.selected_plugins();
        for def in PluginCatalog.for_node_type(NodeType::NeoCli) {
            // The storage plugins follow the storage engine, not a checkbox.
            if matches!(def.id, PluginId::LevelDbStore | PluginId::RocksDbStore) {
                continue;
            }
            let should_enable =
                selected.contains(&def.id) && (def.id != PluginId::RpcServer || node.rpc_port > 0);
            if let Err(error) = state
                .commands
                .set_plugin_enabled(&node.id, def.id, should_enable)
            {
                unapplied.push(format!("{} plugin state not applied: {error}", def.id));
            }
        }
    }

    unapplied
}

/// The flash an operator sees: what happened, plus anything that did not.
fn outcome_message(saved: &str, unapplied: &[String]) -> String {
    if unapplied.is_empty() {
        return saved.to_string();
    }
    format!("{saved} But {}.", unapplied.join("; "))
}

fn render(
    draft: &NodeDraft,
    mode: &EditorMode,
    errors: &FieldErrors,
    context: &EditorContext,
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
        binary = binary_field(draft, errors, &context.installations),
        version = version_field(draft, errors),
        args = args_field(draft, errors),
        p2p = p2p_field(draft, errors),
        rpc_section = rpc_section(draft, errors),
        plugins_section = plugins_section(draft),
        hermes_section = hermes_section(draft),
        signer_section = signer_section(draft, errors, context, mode.current_id()),
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
