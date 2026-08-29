//! The node editor: one form serving both add and edit.
//!
//! Every interaction is a plain form post. Changing the client re-renders with
//! that client's storage choices, "Suggest free ports" asks the planner for a
//! block nothing else claims, and a rejected save comes back with the operator's
//! own text still in the boxes and the reason beside the field it belongs to.

use axum::{
    extract::{Form, Path, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    core::{
        node::{Network, NewNode, NodeType, StorageEngine},
        operations::{EventKind, EventSeverity, NewRuntimeEvent},
    },
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

pub async fn new_form() -> Response {
    render(
        &NodeDraft::blank(),
        &EditorMode::Create,
        &FieldErrors::new(),
    )
}

pub async fn edit_form(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    match find_node(&state, &id) {
        Some(node) => render(
            &NodeDraft::from_node(&node),
            &EditorMode::Edit { id: node.id },
            &FieldErrors::new(),
        ),
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
    // Read the intent before the draft is normalised, which consumes it.
    let wants_client_defaults = form.wants_client_defaults();
    let wants_suggested_ports = form.wants_suggested_ports();
    let draft = form.with_client_defaults();

    if wants_client_defaults {
        return render(&draft, mode, &FieldErrors::new());
    }
    if wants_suggested_ports {
        let suggested = draft
            .suggest_ports(&load_nodes(state), mode.current_id())
            .unwrap_or_else(|| draft.clone());
        return render(&suggested, mode, &FieldErrors::new());
    }

    // `validate` excludes the node being edited by id, so it is given the whole
    // fleet and does the exclusion itself.
    match draft.validate(&load_nodes(state), mode.current_id()) {
        DraftOutcome::Invalid(errors) => render(&draft, mode, &errors),
        DraftOutcome::Valid(input) => match mode {
            EditorMode::Create => save_new(state, draft, input),
            EditorMode::Edit { id } => save_edit(state, draft, id, input),
        },
    }
}

fn save_new(state: &WebState, draft: NodeDraft, input: NewNode) -> Response {
    let name = input.name.clone();
    match state.repository.create_node(input) {
        Ok(node) => {
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
            render(&draft, &EditorMode::Create, &errors)
        }
    }
}

fn save_edit(state: &WebState, draft: NodeDraft, id: &str, input: NewNode) -> Response {
    let name = input.name.clone();
    match state.repository.update_node(id, input) {
        Ok(node) => {
            journal(
                state,
                &node.id,
                &node.name,
                EventKind::NodeUpdated,
                format!("{name} updated"),
            );
            Redirect::to(&redirect_to(id, &format!("{name} saved."))).into_response()
        }
        Err(error) => {
            let mut errors = FieldErrors::new();
            errors.insert("general", error.to_string());
            render(&draft, &EditorMode::Edit { id: id.to_string() }, &errors)
        }
    }
}

fn render(draft: &NodeDraft, mode: &EditorMode, errors: &FieldErrors) -> Response {
    Html(html::layout(
        mode.title(),
        "nodes",
        "",
        &page(draft, mode, errors),
    ))
    .into_response()
}

fn page(draft: &NodeDraft, mode: &EditorMode, errors: &FieldErrors) -> String {
    format!(
        r#"{breadcrumb}
{head}
{summary}
<form method="post" action="{target}">
<div class="panel">
<div class="grid">
{name}{client}{network}{storage}{binary}{version}{args}{rpc}{p2p}{ws}</div>
<div class="form-actions">
<button class="primary" type="submit">{save}</button>
<button type="submit" name="suggest" value="1">Suggest free ports</button>
<button type="submit" name="client" value="1">Apply client defaults</button>
<span class="spacer"></span>
<a class="btn" href="{cancel}">Cancel</a>
</div>
</div>
</form>"#,
        breadcrumb = html::breadcrumb(&[("Nodes", "/nodes"), (mode.title(), "")]),
        head = html::page_head(
            mode.title(),
            "Register a node so the workbench can manage its config, ports and lifecycle.",
            "",
        ),
        summary = summary(errors),
        target = mode.post_target(),
        cancel = mode.cancel_target(),
        save = mode.submit_label(),
        name = name_field(draft, errors),
        client = client_field(draft, errors),
        network = network_field(draft, errors),
        storage = storage_field(draft, errors),
        binary = binary_field(draft, errors),
        version = version_field(draft, errors),
        args = args_field(draft, errors),
        rpc = port_field(draft, errors),
        p2p = p2p_field(draft, errors),
        ws = ws_field(draft, errors),
    )
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

fn error_for<'a>(errors: &'a FieldErrors, key: &str) -> Option<&'a str> {
    errors.get(key).map(String::as_str)
}

fn name_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
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

fn client_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
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

fn network_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
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

/// Storage is only a choice on the clients that offer one. Rendering a dropdown
/// that cannot change anything hides the cases where it can.
fn storage_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
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
    // Both engines stay offered even where one is invalid, so a hand-edited post
    // produces a precise error rather than a value that silently disappeared.
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

fn binary_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    html::TextField {
        label: "Node binary",
        name: "binary_path",
        value: &draft.binary_path,
        error: error_for(errors, "binary_path"),
        help: Some("Absolute path, or one installed from the runtime catalogue."),
        monospace: true,
        full_width: true,
        placeholder: Some("/opt/neo/neo-go"),
    }
    .render()
}

fn version_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    html::TextField {
        label: "Runtime version",
        name: "runtime_version",
        value: &draft.runtime_version,
        error: error_for(errors, "runtime_version"),
        help: Some("Blank means latest."),
        monospace: true,
        ..html::TextField::default()
    }
    .render()
}

fn args_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
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

fn port_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    html::TextField {
        label: "RPC port",
        name: "rpc_port",
        value: &draft.rpc_port,
        error: error_for(errors, "rpc_port"),
        help: Some("Must be free on this host and unused by another node."),
        ..html::TextField::default()
    }
    .render()
}

fn p2p_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    html::TextField {
        label: "P2P port",
        name: "p2p_port",
        value: &draft.p2p_port,
        error: error_for(errors, "p2p_port"),
        ..html::TextField::default()
    }
    .render()
}

fn ws_field(draft: &NodeDraft, errors: &FieldErrors) -> String {
    html::TextField {
        label: "WebSocket port",
        name: "ws_port",
        value: &draft.ws_port,
        error: error_for(errors, "ws_port"),
        help: Some("Leave blank if the node exposes none."),
        ..html::TextField::default()
    }
    .render()
}

fn labels<T: std::fmt::Display, const N: usize>(values: [T; N]) -> Vec<String> {
    values.iter().map(T::to_string).collect()
}

fn load_nodes(state: &WebState) -> Vec<NodeConfig> {
    state.repository.list_nodes().unwrap_or_default()
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
    let _ = state.repository.record_event(NewRuntimeEvent {
        node_id: Some(node_id.to_string()),
        node_name: Some(node_name.to_string()),
        kind,
        severity: EventSeverity::Info,
        message,
    });
}
