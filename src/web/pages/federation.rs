//! Federation: the peer NeoNexus deployments this workspace watches, and the
//! last probe recorded for each. The page reads the stored probe history rather
//! than reaching out on a page load — a browser refresh must not become a
//! traffic event against someone else's fleet.

use axum::{
    extract::{Path, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::core::operations::{
    filter_remote_server_profiles, RemoteProbeStatus, RemoteServerProfile,
    RemoteServerProfileFilter,
};

use super::super::{html, time, WebState};

const PROBE_WINDOW: usize = 12;

#[derive(Default, Deserialize)]
pub struct FederationQuery {
    #[serde(default)]
    enabled: String,
    #[serde(default)]
    q: String,
}

pub async fn federation(
    State(state): State<WebState>,
    RawQuery(flash): RawQuery,
    Query(params): Query<FederationQuery>,
) -> Response {
    let body = match render_body(&state, &params) {
        Ok(body) => body,
        Err(error) => html::note(&format!("failed to load federation profiles: {error}")),
    };
    Html(html::layout(
        "Federation",
        "federation",
        html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

fn render_body(state: &WebState, params: &FederationQuery) -> anyhow::Result<String> {
    let profiles = state.workspace.list_remote_servers()?;
    let filter = RemoteServerProfileFilter::new(tri_state(&params.enabled), params.q.trim());
    let visible = filter_remote_server_profiles(&profiles, &filter);
    Ok(format!(
        r#"<h1>Federation</h1>
{tiles}
{filters}
{table}
{add_form}"#,
        add_form = add_form(),
        tiles = html::cards(&[
            ("Servers", profiles.len().to_string()),
            (
                "Enabled",
                profiles
                    .iter()
                    .filter(|profile| profile.enabled)
                    .count()
                    .to_string(),
            ),
            (
                "Healthy",
                count_status(state, &profiles, RemoteProbeStatus::Healthy)?,
            ),
            (
                "Unreachable",
                count_status(state, &profiles, RemoteProbeStatus::Unreachable)?,
            ),
        ]),
        filters = html::typed_filter_form(
            "/federation",
            &[],
            &[
                html::FilterControl::Select {
                    label: "Status",
                    name: "enabled",
                    selected: &params.enabled,
                    options: &[("", "All servers"), ("yes", "Enabled"), ("no", "Disabled")],
                },
                html::FilterControl::Search {
                    label: "Search",
                    name: "q",
                    value: &params.q,
                    placeholder: "Server, URL, or profile",
                },
            ],
        ),
        table = profile_table(state, &visible)?,
    ))
}

fn count_status(
    state: &WebState,
    profiles: &[RemoteServerProfile],
    wanted: RemoteProbeStatus,
) -> anyhow::Result<String> {
    let mut matching = 0;
    for profile in profiles {
        if let Some(probe) = state.workspace.latest_remote_server_probe(&profile.id)? {
            if probe.status == wanted {
                matching += 1;
            }
        }
    }
    Ok(matching.to_string())
}

fn profile_table(state: &WebState, profiles: &[RemoteServerProfile]) -> anyhow::Result<String> {
    if profiles.is_empty() {
        return Ok(html::empty_state(
            "No federation servers",
            "A federation server is another NeoNexus workspace whose fleet summary this one \
             polls. Add its base URL below.",
            "",
        ));
    }
    let mut rows = Vec::new();
    for profile in profiles {
        let probe = state.workspace.latest_remote_server_probe(&profile.id)?;
        rows.push(html::row(&[
            html::cell(&profile.name),
            html::cell(&profile.base_url),
            html::raw_cell(&enabled_badge(profile.enabled)),
            html::raw_cell(
                &probe
                    .as_ref()
                    .map(|record| status_badge(record.status))
                    .unwrap_or_else(|| html::status_badge("Unknown")),
            ),
            html::raw_cell(&time::time_cell(
                probe.as_ref().map(|record| record.checked_at_unix),
            )),
            html::cell(
                &probe
                    .as_ref()
                    .and_then(|record| record.total_nodes)
                    .map(|nodes| nodes.to_string())
                    .unwrap_or_else(|| "—".to_string()),
            ),
            html::cell(
                &probe
                    .as_ref()
                    .and_then(|record| record.running_nodes)
                    .map(|nodes| nodes.to_string())
                    .unwrap_or_else(|| "—".to_string()),
            ),
            html::cell(
                &probe
                    .as_ref()
                    .map(|record| record.message.clone())
                    .unwrap_or_default(),
            ),
            html::raw_cell(&toggle_form(profile)),
            html::raw_cell(&format!(
                r#"<a class="btn small" href="/federation/{id}/edit">Edit</a> <a class="btn small" href="/federation/{id}/probes">History</a><form method="post" action="/federation/{id}/delete" style="display:inline; margin-left:4px;"><button type="submit" class="btn small danger" title="Remove this server and its probe history">Remove</button></form>"#,
                id = html::escape(&profile.id)
            )),
        ]));
    }
    Ok(html::table(
        &[
            "Server",
            "Base URL",
            "Status",
            "Last probe",
            "Checked",
            "Nodes",
            "Running",
            "Message",
            "Control",
            "History",
        ],
        &rows,
    ))
}

/// The form that was missing.
///
/// Every neighbouring entity in this console has an in-page create form; this
/// page had a list, a toggle, and a sentence telling the operator to use the
/// Rust API.
fn add_form() -> String {
    r#"<div class="panel" style="margin-top: 18px; padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
  <h3 style="margin-top: 0;">Add a federation server</h3>
  <p class="muted" style="font-size: 12px;">
    Another NeoNexus workspace whose fleet summary this one polls. It publishes counts only —
    node names, heights and peers stay behind its own session.
  </p>
  <form method="post" action="/federation/new" class="filters">
    <label class="field"><span>Name</span><input name="name" placeholder="frankfurt" required></label>
    <label class="field"><span>Base URL</span><input name="base_url" placeholder="https://neonexus.example.com" required></label>
    <label class="field"><span>Description</span><input name="description" placeholder="optional"></label>
    <label class="field"><span>Poll it</span><input type="checkbox" name="enabled" value="on" checked></label>
    <button type="submit">Add</button>
  </form>
</div>"#
    .to_string()
}

/// The edit form for one federation server.
///
/// The route that saves it — `POST /federation/{id}` — existed and was
/// complete, but nothing in this console ever pointed at it: an operator
/// could add a peer and then not change a single thing about it. The form
/// reuses the same fields as the add form so the two cannot drift apart.
fn edit_form(profile: &RemoteServerProfile) -> String {
    format!(
        r#"<div class="panel" style="margin-top: 18px; padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
  <h3 style="margin-top: 0;">Edit {name}</h3>
  <form method="post" action="/federation/{id}" class="filters">
    <label class="field"><span>Name</span><input name="name" value="{name}" required></label>
    <label class="field"><span>Base URL</span><input name="base_url" value="{base_url}" required></label>
    <label class="field"><span>Description</span><input name="description" value="{description}"></label>
    <label class="field"><span>Poll it</span><input type="checkbox" name="enabled" value="on"{checked}></label>
    <button type="submit">Save</button>
    <a class="btn" href="/federation">Cancel</a>
  </form>
</div>"#,
        name = html::escape(&profile.name),
        id = html::escape(&profile.id),
        base_url = html::escape(&profile.base_url),
        description = html::escape(&profile.description),
        checked = if profile.enabled { " checked" } else { "" },
    )
}

/// The edit page for one federation server.
///
/// A GET, so the operator can reach it from the list, read what is stored
/// there, and change one thing without retyping the rest.
pub async fn edit(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let body = (|| -> anyhow::Result<String> {
        let profile = state
            .workspace
            .list_remote_servers()?
            .into_iter()
            .find(|profile| profile.id == id)
            .ok_or_else(|| anyhow::anyhow!("federation server {id} was not found"))?;
        Ok(format!(
            r#"<h1>Edit federation server</h1>
{back}
{form}"#,
            back = html::breadcrumb(&[("Federation", "/federation"), ("Edit", "")]),
            form = edit_form(&profile),
        ))
    })();
    let body = match body {
        Ok(body) => body,
        Err(error) => html::notice("danger", &format!("{error}")),
    };
    Html(html::layout("Federation", "federation", "", &body)).into_response()
}

fn toggle_form(profile: &RemoteServerProfile) -> String {
    let label = if profile.enabled { "Disable" } else { "Enable" };
    // Turning a peer off takes it out of the federation; turning it back on
    // puts it back. The two are not the same act and must not look the same.
    if profile.enabled {
        html::danger_control_form(&format!("/federation/{}/toggle", profile.id), &[], label)
    } else {
        html::control_form(&format!("/federation/{}/toggle", profile.id), &[], label)
    }
}

pub async fn toggle(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let profile = state
            .workspace
            .list_remote_servers()?
            .into_iter()
            .find(|profile| profile.id == id)
            .ok_or_else(|| anyhow::anyhow!("federation server {id} was not found"))?;
        let updated = state
            .commands
            .set_remote_server_enabled(&profile.id, !profile.enabled)?;
        Ok(format!(
            "{} {}",
            updated.name,
            if updated.enabled {
                "enabled"
            } else {
                "disabled"
            }
        ))
    })();
    let flash = match outcome {
        Ok(message) => html::Flash::ok(message),
        Err(error) => html::Flash::danger(format!("not changed: {error}")),
    };
    Redirect::to(&flash.redirect("/federation")).into_response()
}

/// The recorded probe trail for one server, newest first.
pub async fn probes(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let body = (|| -> anyhow::Result<String> {
        let profile = state
            .workspace
            .list_remote_servers()?
            .into_iter()
            .find(|profile| profile.id == id);
        let Some(profile) = profile else {
            return Ok(html::note(&format!(
                "Federation server {id} is no longer configured."
            )));
        };
        let history = state
            .workspace
            .list_remote_server_probes(&profile.id, PROBE_WINDOW)?;
        let rows = history
            .iter()
            .map(|record| {
                html::row(&[
                    html::raw_cell(&time::time_cell(Some(record.checked_at_unix))),
                    html::raw_cell(&status_badge(record.status)),
                    html::cell(&number(record.total_nodes)),
                    html::cell(&number(record.running_nodes)),
                    html::cell(&number(record.syncing_nodes)),
                    html::cell(&number(record.error_nodes)),
                    html::cell(&number(record.total_blocks)),
                    html::cell(&number(record.total_peers)),
                    html::cell(&record.message),
                ])
            })
            .collect::<Vec<_>>();
        Ok(format!(
            r#"<h1>{name} probe history</h1>
{back}
{table}"#,
            name = html::escape(&profile.name),
            back = html::breadcrumb(&[("Federation", "/federation"), ("Probe history", "")]),
            table = if rows.is_empty() {
                html::empty_state(
                    "No probes recorded yet",
                    "This federation server has not been polled yet. Probe history appears here \
                     once the federation monitor checks it.",
                    "",
                )
            } else {
                html::table(
                    &[
                        "Checked", "Status", "Nodes", "Running", "Syncing", "Error", "Blocks",
                        "Peers", "Message",
                    ],
                    &rows,
                )
            },
        ))
    })();
    let body = body.unwrap_or_else(|error| html::note(&error.to_string()));
    Html(html::layout("Federation", "federation", "", &body)).into_response()
}

fn number(value: Option<u64>) -> String {
    value.map_or_else(|| "—".to_string(), |value| value.to_string())
}

fn status_badge(status: RemoteProbeStatus) -> String {
    let class = match status {
        RemoteProbeStatus::Healthy => "badge running",
        RemoteProbeStatus::Degraded => "badge starting",
        RemoteProbeStatus::Unreachable => "badge error",
        RemoteProbeStatus::Disabled => "badge stopped",
    };
    format!(
        r#"<span class="{class}">{}</span>"#,
        html::escape(status.label())
    )
}

fn enabled_badge(enabled: bool) -> String {
    let class = if enabled {
        "badge running"
    } else {
        "badge stopped"
    };
    format!(
        r#"<span class="{class}">{}</span>"#,
        if enabled { "enabled" } else { "disabled" }
    )
}

fn tri_state(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" => Some(true),
        "no" | "false" => Some(false),
        _ => None,
    }
}

/// Register, edit or forget a federation peer.
///
/// `create_remote_server` / `update_remote_server` / `delete_remote_server`
/// were complete and called only from tests; the only production insert was
/// backup import. So the page listed peers, let an operator toggle them, and
/// told them to "add one through the Rust API" — while every neighbouring
/// entity in this console has an in-page create form.
#[derive(serde::Deserialize)]
pub struct RemoteServerForm {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub enabled: String,
}

impl RemoteServerForm {
    fn to_input(&self) -> crate::federation::NewRemoteServerProfile {
        crate::federation::NewRemoteServerProfile {
            name: self.name.trim().to_string(),
            base_url: self.base_url.trim().to_string(),
            description: self.description.trim().to_string(),
            // Absent checkbox means unchecked; a browser posts nothing for one.
            enabled: matches!(
                self.enabled.trim().to_ascii_lowercase().as_str(),
                "on" | "1" | "true" | "yes"
            ),
        }
    }
}

pub async fn create(
    State(state): State<WebState>,
    axum::Form(form): axum::Form<RemoteServerForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let profile = state.commands.create_remote_server(form.to_input())?;
        journal(
            &state,
            crate::events::EventKind::RemoteServerCreated,
            format!(
                "federation server {} added ({})",
                profile.name, profile.base_url
            ),
        );
        Ok(format!("added {}", profile.name))
    })();
    redirect(outcome)
}

pub async fn update(
    State(state): State<WebState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::Form(form): axum::Form<RemoteServerForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let profile = state.commands.update_remote_server(&id, form.to_input())?;
        journal(
            &state,
            crate::events::EventKind::RemoteServerUpdated,
            format!(
                "federation server {} updated ({})",
                profile.name, profile.base_url
            ),
        );
        Ok(format!("updated {}", profile.name))
    })();
    match outcome {
        Ok(message) => {
            let flash = html::Flash::ok(message);
            Redirect::to(&flash.redirect("/federation")).into_response()
        }
        Err(error) => {
            // The operator typed a base URL the domain would not take. Send
            // them back to the form with what they wrote still in it, so the
            // fix is one edit rather than a retype.
            let body = format!(
                r#"<h1>Edit federation server</h1>
{back}
{danger}
{form}"#,
                back = html::breadcrumb(&[("Federation", "/federation"), ("Edit", "")]),
                danger = html::notice("danger", &format!("not saved: {error}")),
                form = edit_form_from_form(&id, &form),
            );
            Html(html::layout("Federation", "federation", "", &body)).into_response()
        }
    }
}

/// The edit form filled with what the operator submitted, rather than with
/// what is stored.
///
/// The two differ exactly when the save failed, which is the only time this
/// form is rendered.
fn edit_form_from_form(id: &str, form: &RemoteServerForm) -> String {
    let enabled = matches!(
        form.enabled.trim().to_ascii_lowercase().as_str(),
        "on" | "1" | "true" | "yes"
    );
    format!(
        r#"<div class="panel" style="margin-top: 18px; padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
  <h3 style="margin-top: 0;">Edit federation server</h3>
  <form method="post" action="/federation/{id}" class="filters">
    <label class="field"><span>Name</span><input name="name" value="{name}" required></label>
    <label class="field"><span>Base URL</span><input name="base_url" value="{base_url}" required></label>
    <label class="field"><span>Description</span><input name="description" value="{description}"></label>
    <label class="field"><span>Poll it</span><input type="checkbox" name="enabled" value="on"{checked}></label>
    <button type="submit">Save</button>
    <a class="btn" href="/federation">Cancel</a>
  </form>
</div>"#,
        id = html::escape(id),
        name = html::escape(form.name.trim()),
        base_url = html::escape(form.base_url.trim()),
        description = html::escape(form.description.trim()),
        checked = if enabled { " checked" } else { "" },
    )
}

pub async fn delete(
    State(state): State<WebState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let name = state
            .workspace
            .list_remote_servers()?
            .into_iter()
            .find(|profile| profile.id == id)
            .map_or_else(|| id.clone(), |profile| profile.name);
        state.commands.delete_remote_server(&id)?;
        journal(
            &state,
            crate::events::EventKind::RemoteServerDeleted,
            format!("federation server {name} removed, with its probe history"),
        );
        Ok(format!("removed {name}"))
    })();
    redirect(outcome)
}

fn journal(state: &WebState, kind: crate::events::EventKind, message: String) {
    let _ = state.commands.record_event(crate::events::NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind,
        severity: crate::events::EventSeverity::Info,
        message,
    });
}

fn redirect(outcome: anyhow::Result<String>) -> Response {
    let flash = match outcome {
        Ok(message) => html::Flash::ok(message),
        Err(error) => html::Flash::danger(format!("federation change failed: {error:#}")),
    };
    axum::response::Redirect::to(&flash.redirect("/federation")).into_response()
}
