//! Hermes and other companion processes share the core lifecycle service.
use super::super::{html, WebState};
use crate::core::agents::{self, AgentKind, AgentProfile, AgentRecord};
use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct AgentQuery {
    #[serde(default)]
    edit: String,
    #[serde(default)]
    flash: String,
}

pub async fn page(State(state): State<WebState>, Query(query): Query<AgentQuery>) -> Response {
    let body = match state.repository.list_agents() {
        Ok(records) => render(&records, &query.edit),
        Err(error) => html::note(&error.to_string()),
    };
    Html(html::layout("Agents", "agents", &query.flash, &body)).into_response()
}

fn render(records: &[AgentRecord], edit: &str) -> String {
    let rows = records.iter().map(|record| {
        let p = &record.profile;
        let status = format!("{:?}", record.status);
        let health = match record.healthy { Some(true) => "Healthy", Some(false) => "Unhealthy", None => "Not verified" };
        let mut controls = format!(r#"<a class="btn" href="/agents?edit={}">Edit / review version</a> <a class="btn" href="/agents/{}/logs">Logs</a>"#, html::escape(&p.id), html::escape(&p.id));
        for (action, label) in [("start", "Start"), ("stop", "Stop"), ("restart", "Restart"), ("delete", "Delete profile")] {
            controls.push_str(&format!(r#"<form method="post" action="/agents/{}/{action}" style="display:inline"><button>{label}</button></form> "#, html::escape(&p.id)));
        }
        if record.pid.is_some() {
            controls.push_str(&format!(r#"<form method="post" action="/agents/{}/forget-stale" style="display:inline"><button title="Clear only an absent or mismatched process identity; never signals a process">Clear stale PID</button></form> "#, html::escape(&p.id)));
        }
        html::row(&[html::cell(&p.name), html::cell(&format!("{:?}", p.kind)), html::cell(p.node_id.as_deref().unwrap_or("Workspace")),
            html::cell(&p.version), html::cell(&status), html::cell(&record.pid.map(|pid| pid.to_string()).unwrap_or_else(|| "—".into())),
            html::cell(health), html::cell(&format!("{}/3{}", record.restart_attempts, if record.restart_after.is_some() { " pending" } else { "" })), format!("<td>{controls}</td>")])
    }).collect::<Vec<_>>();
    format!("<h1>Agents</h1>{}<h2>{}</h2>{}{}",
        html::table(&["Name", "Kind", "Node", "Declared version", "State", "PID", "Health", "Restarts", "Controls"], &rows),
        if edit.is_empty() { "Register companion" } else { "Review stopped companion" },
        html::note("For Nous Research Hermes: select the installation's Python interpreter, use the hermes-agent source root as working directory, select your profile's config.yaml, and leave arguments empty. NeoNexus runs its foreground gateway with an isolated HERMES_HOME. Other companions use your argument array. Saving pins executable and configuration fingerprints; stop and review before changes. Credentials stay in the companion's configuration file."),
        editor(records.iter().find(|record| record.profile.id == edit).map(|record| &record.profile)))
}

fn editor(profile: Option<&AgentProfile>) -> String {
    let value = |f: fn(&AgentProfile) -> String| profile.map(f).unwrap_or_default();
    let mut body = format!(
        r#"<form method="post" action="/agents/save"><input type="hidden" name="id" value="{}">"#,
        html::escape(&value(|p| p.id.clone()))
    );
    for (name, label, text) in [
        ("name", "Name", value(|p| p.name.clone())),
        (
            "kind",
            "Kind: hermes, signer, plugin, sidecar",
            profile
                .map(|p| format!("{:?}", p.kind).to_lowercase())
                .unwrap_or_else(|| "hermes".into()),
        ),
        (
            "node_id",
            "Associated node ID (optional)",
            value(|p| p.node_id.clone().unwrap_or_default()),
        ),
        ("version", "Declared version", value(|p| p.version.clone())),
        (
            "binary_path",
            "Installed executable (absolute path)",
            value(|p| p.binary_path.display().to_string()),
        ),
        (
            "working_dir",
            "Working directory (absolute path)",
            value(|p| p.working_dir.display().to_string()),
        ),
        (
            "config_path",
            "Configuration file (optional absolute path)",
            value(|p| {
                p.config_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default()
            }),
        ),
        (
            "health_url",
            "HTTP health URL (optional)",
            value(|p| p.health_url.clone().unwrap_or_default()),
        ),
    ] {
        body.push_str(&format!(
            r#"<label>{label}<input name="{name}" value="{}"></label>"#,
            html::escape(&text)
        ));
    }
    let arguments = profile
        .and_then(|p| serde_json::to_string(&p.args).ok())
        .unwrap_or_else(|| "[]".into());
    body.push_str(&format!(r#"<label>Arguments (JSON array, each item is one argument)<textarea name="args">{}</textarea></label><p>Available references: {{config}}, {{node_id}}, {{rpc_url}}. No secret values in arguments.</p><label><input type="checkbox" name="auto_restart" value="true" {}>Restart after crashes, at most 3 attempts (5 / 10 / 20 seconds)</label><button>Save reviewed profile</button></form>"#, html::escape(&arguments), if profile.is_some_and(|p| p.auto_restart) { "checked" } else { "" }));
    body
}

#[derive(Deserialize)]
pub struct AgentForm {
    #[serde(default)]
    id: String,
    name: String,
    kind: String,
    version: String,
    #[serde(default)]
    node_id: String,
    binary_path: String,
    working_dir: String,
    #[serde(default)]
    args: String,
    #[serde(default)]
    config_path: String,
    #[serde(default)]
    health_url: String,
    #[serde(default)]
    auto_restart: String,
}

pub async fn save(State(state): State<WebState>, Form(form): Form<AgentForm>) -> Response {
    let result = (|| {
        let kind = match form.kind.trim() {
            "hermes" => AgentKind::Hermes,
            "signer" => AgentKind::Signer,
            "plugin" => AgentKind::Plugin,
            "sidecar" => AgentKind::Sidecar,
            _ => anyhow::bail!("invalid agent kind"),
        };
        let optional = |value: String| (!value.trim().is_empty()).then(|| value.trim().to_string());
        agents::save(
            &state.engine_state(),
            AgentProfile {
                id: form.id,
                name: form.name,
                kind,
                node_id: optional(form.node_id),
                version: form.version,
                binary_path: form.binary_path.trim().into(),
                working_dir: form.working_dir.trim().into(),
                args: serde_json::from_str(&form.args)?,
                config_path: optional(form.config_path).map(Into::into),
                health_url: optional(form.health_url),
                auto_restart: form.auto_restart == "true",
                binary_sha256: String::new(),
                config_sha256: None,
            },
        )
    })();
    back(result, "Agent profile saved")
}

pub async fn control(
    State(state): State<WebState>,
    Path((id, action)): Path<(String, String)>,
) -> Response {
    let engine = state.engine_state();
    let result = match action.as_str() {
        "start" => agents::start(&engine, &id),
        "stop" => agents::stop(&engine, &id),
        "restart" => agents::stop(&engine, &id).and_then(|()| agents::start(&engine, &id)),
        "delete" => agents::delete(&engine, &id),
        "forget-stale" => agents::forget_stale(&engine, &id),
        _ => Err(anyhow::anyhow!("unknown agent action")),
    };
    back(result, "Agent action completed")
}

fn back(result: anyhow::Result<()>, success: &str) -> Response {
    let message = result
        .err()
        .map_or_else(|| success.to_string(), |error| error.to_string());
    Redirect::to(&format!("/agents{}", html::flash_query(&message))).into_response()
}

pub async fn logs(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let body = match state.repository.list_agents() {
        Ok(records) if records.iter().any(|record| record.profile.id == id) => {
            match crate::core::runtime::LogReader::snapshot(
                state.data_dir.join("logs").join(format!("agent-{id}.log")),
                64 * 1024,
            ) {
                Ok(snapshot) => html::text_block(&crate::redaction::redact_sensitive_text(
                    &snapshot.lines.join("\n"),
                )),
                Err(error) => html::note(&error.to_string()),
            }
        }
        _ => html::note("Agent not found"),
    };
    Html(html::layout(
        "Agent logs",
        "agents",
        "",
        &format!("<h1>Agent logs</h1>{body}"),
    ))
    .into_response()
}
