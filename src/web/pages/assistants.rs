//! Connect existing Hermes instances to scoped node tools.
use super::super::{html, WebState};
use crate::core::{
    agents::{AgentKind, AgentRecord},
    assistants::{self, AssistantDraft, AssistantProfile},
};
use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Default, Deserialize)]
pub struct AssistantQuery {
    #[serde(default)]
    edit: String,
    #[serde(default)]
    flash: String,
}

pub async fn page(State(state): State<WebState>, Query(query): Query<AssistantQuery>) -> Response {
    let result = (|| -> anyhow::Result<String> {
        let profiles = state.repository.list_assistants()?;
        let agents = state.repository.list_agents()?;
        let nodes = state.repository.list_nodes()?;
        Ok(render(&profiles, &agents, &nodes, &query.edit))
    })();
    let body = result.unwrap_or_else(|error| html::note(&error.to_string()));
    Html(html::layout(
        "Assistants",
        "assistants",
        &query.flash,
        &body,
    ))
    .into_response()
}

fn render(
    profiles: &[AssistantProfile],
    agents: &[AgentRecord],
    nodes: &[crate::types::NodeConfig],
    edit: &str,
) -> String {
    let rows: Vec<_> = profiles.iter().map(|profile| {
        let instance = agents.iter().find(|agent| agent.profile.id == profile.agent_id).map_or(profile.agent_id.as_str(), |agent| agent.profile.name.as_str());
        let scope = if profile.all_nodes { "All current and future nodes".into() } else {
            profile.node_ids.iter().map(|id| nodes.iter().find(|node| &node.id == id).map_or(id.as_str(), |node| node.name.as_str())).collect::<Vec<_>>().join(", ")
        };
        html::row(&[html::cell(&profile.name), html::cell(instance), html::cell(&scope),
            html::cell(if profile.can_operate { "Monitor + start / stop / restart" } else { "Monitor only" }),
            html::cell(if profile.enabled { "Access enabled" } else { "Revoked" }),
            format!(r#"<td><a class="btn" href="/assistants?edit={}">Review / reconnect</a> <form method="post" action="/assistants/{}/revoke" style="display:inline"><button>Revoke access</button></form></td>"#, html::escape(&profile.id), html::escape(&profile.id))])
    }).collect();
    let selected = profiles.iter().find(|profile| profile.id == edit);
    let mut body = format!("<h1>Assistants</h1>{}{}<h2>{}</h2>",
        html::note("Connect the Hermes instance you already use. Telegram and other conversation channels remain configured in Hermes. These credentials expose only the selected NeoNexus node tools; wallet and signer secrets are not available. Stop Hermes before connecting or changing permissions, then start it from Agents when ready."),
        html::table(&["Name", "Hermes instance", "Node scope", "Permissions", "Access", "Actions"], &rows),
        if selected.is_some() { "Review connection" } else { "Connect Hermes" });
    let hermes: Vec<_> = agents
        .iter()
        .filter(|agent| agent.profile.kind == AgentKind::Hermes)
        .collect();
    if hermes.is_empty() {
        body.push_str(r#"<p>Register your existing Hermes installation on <a href="/agents">Agents</a> first.</p>"#);
        return body;
    }
    body.push_str(&format!(r#"<form method="post" action="/assistants/connect"><input type="hidden" name="id" value="{}"><label>Connection name<input name="name" required maxlength="120" value="{}"></label><label>Hermes instance<select name="agent_id">"#,
        html::escape(selected.map_or("", |profile| profile.id.as_str())), html::escape(selected.map_or("", |profile| profile.name.as_str()))));
    for agent in &hermes {
        if selected.is_some_and(|profile| profile.agent_id != agent.profile.id) {
            continue;
        }
        body.push_str(&format!(
            r#"<option value="{}" {}>{} ({:?})</option>"#,
            html::escape(&agent.profile.id),
            if selected.is_some_and(|profile| profile.agent_id == agent.profile.id) {
                "selected"
            } else {
                ""
            },
            html::escape(&agent.profile.name),
            agent.status
        ));
    }
    let endpoint = selected
        .and_then(|profile| {
            agents
                .iter()
                .find(|agent| agent.profile.id == profile.agent_id)
                .and_then(|agent| agent.profile.config_path.as_deref())
                .and_then(|path| {
                    assistants::hermes_config::configured_endpoint(
                        path,
                        &format!("neonexus_{}", profile.id),
                    )
                    .ok()
                    .flatten()
                })
        })
        .unwrap_or_default();
    body.push_str(&format!(r#"</select></label><label>NeoNexus MCP address<input id="assistant-endpoint" name="endpoint" type="url" value="{}" required placeholder="http://127.0.0.1:8080/mcp"></label><p>Use the address this Hermes process can reach. HTTPS or loopback HTTP is required.</p><fieldset><legend>Node scope</legend>"#, html::escape(&endpoint)));
    body.push_str(&format!(r#"<label><input type="checkbox" name="all_nodes" value="true" {}>All current and future nodes</label>"#, if selected.is_some_and(|profile| profile.all_nodes) { "checked" } else { "" }));
    for node in nodes {
        body.push_str(&format!(
            r#"<label><input type="checkbox" name="node_{}" value="true" {}>{}</label>"#,
            html::escape(&node.id),
            if selected.is_some_and(|profile| profile.node_ids.contains(&node.id)) {
                "checked"
            } else {
                ""
            },
            html::escape(&node.name)
        ));
    }
    body.push_str(&format!(r#"</fieldset><label><input type="checkbox" name="can_operate" value="true" {}>Allow node start, stop and restart</label><p>Unchecked means monitoring only. Saving rotates this connection's credential and keeps backups of the Hermes files. No channel messages are sent.</p><button>Connect and save permissions</button></form><script>const e=document.getElementById('assistant-endpoint');if(e&&!e.value)e.value=window.location.origin+'/mcp';</script>"#,
        if selected.is_some_and(|profile| profile.can_operate) { "checked" } else { "" }));
    body
}

pub async fn connect(
    State(state): State<WebState>,
    Form(mut fields): Form<HashMap<String, String>>,
) -> Response {
    let field =
        |fields: &mut HashMap<String, String>, key: &str| fields.remove(key).unwrap_or_default();
    let endpoint = field(&mut fields, "endpoint");
    let draft = AssistantDraft {
        id: field(&mut fields, "id"),
        name: field(&mut fields, "name"),
        agent_id: field(&mut fields, "agent_id"),
        all_nodes: field(&mut fields, "all_nodes") == "true",
        can_operate: field(&mut fields, "can_operate") == "true",
        node_ids: fields
            .into_iter()
            .filter_map(|(key, value)| {
                (value == "true")
                    .then(|| key.strip_prefix("node_").map(str::to_string))
                    .flatten()
            })
            .collect(),
    };
    let result = assistants::connect_hermes(&state.engine_state(), draft, endpoint.trim());
    back(result.map(|_| ()), "Hermes connection configured. Configuration backups retained; start Hermes from Agents when ready.")
}

pub async fn revoke(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    back(
        assistants::revoke(&state.repository, &id),
        "Assistant access revoked immediately. Existing Hermes channels are unchanged.",
    )
}

fn back(result: anyhow::Result<()>, success: &str) -> Response {
    let message = result
        .err()
        .map_or_else(|| success.to_string(), |error| error.to_string());
    Redirect::to(&format!("/assistants{}", html::flash_query(&message))).into_response()
}
