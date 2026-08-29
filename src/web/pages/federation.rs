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

use super::super::{html, WebState};

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
        &html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

fn render_body(state: &WebState, params: &FederationQuery) -> anyhow::Result<String> {
    let profiles = state.repository.list_remote_servers()?;
    let filter = RemoteServerProfileFilter::new(tri_state(&params.enabled), params.q.trim());
    let visible = filter_remote_server_profiles(&profiles, &filter);
    Ok(format!(
        r#"<h1>Federation</h1>
{tiles}
{filters}
{table}"#,
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
        filters = html::filter_form(
            "/federation",
            &[("enabled", &params.enabled), ("q", &params.q)],
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
        if let Some(probe) = state.repository.latest_remote_server_probe(&profile.id)? {
            if probe.status == wanted {
                matching += 1;
            }
        }
    }
    Ok(matching.to_string())
}

fn profile_table(state: &WebState, profiles: &[RemoteServerProfile]) -> anyhow::Result<String> {
    if profiles.is_empty() {
        return Ok(html::note(
            "No federation servers are configured. Add one through the Rust API or restore a backup that carries them.",
        ));
    }
    let mut rows = Vec::new();
    for profile in profiles {
        let probe = state.repository.latest_remote_server_probe(&profile.id)?;
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
            html::cell(
                &probe
                    .as_ref()
                    .map(|record| record.checked_at_unix.to_string())
                    .unwrap_or_else(|| "never".to_string()),
            ),
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
                r#"<a class="btn" href="/federation/{}/probes">History</a>"#,
                html::escape(&profile.id)
            )),
        ]));
    }
    Ok(html::table(
        &[
            "Server",
            "Base URL",
            "Status",
            "Last probe",
            "Checked (unix)",
            "Nodes",
            "Running",
            "Message",
            "Control",
            "History",
        ],
        &rows,
    ))
}

fn toggle_form(profile: &RemoteServerProfile) -> String {
    let label = if profile.enabled { "Disable" } else { "Enable" };
    html::control_form(&format!("/federation/{}/toggle", profile.id), &[], label)
}

pub async fn toggle(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let profile = state
            .repository
            .list_remote_servers()?
            .into_iter()
            .find(|profile| profile.id == id)
            .ok_or_else(|| anyhow::anyhow!("federation server {id} was not found"))?;
        let updated = state
            .repository
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
    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("not changed: {error}"),
    };
    Redirect::to(&format!(
        "/federation?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}

/// The recorded probe trail for one server, newest first.
pub async fn probes(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let body = (|| -> anyhow::Result<String> {
        let profile = state
            .repository
            .list_remote_servers()?
            .into_iter()
            .find(|profile| profile.id == id);
        let Some(profile) = profile else {
            return Ok(html::note(&format!(
                "Federation server {id} is no longer configured."
            )));
        };
        let history = state
            .repository
            .list_remote_server_probes(&profile.id, PROBE_WINDOW)?;
        let rows = history
            .iter()
            .map(|record| {
                html::row(&[
                    html::cell(&record.checked_at_unix.to_string()),
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
<p><a href="/federation">&larr; All servers</a></p>
{table}"#,
            name = html::escape(&profile.name),
            table = if rows.is_empty() {
                html::note("No probes have been recorded for this server yet.")
            } else {
                html::table(
                    &[
                        "Checked (unix)",
                        "Status",
                        "Nodes",
                        "Running",
                        "Syncing",
                        "Error",
                        "Blocks",
                        "Peers",
                        "Message",
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
