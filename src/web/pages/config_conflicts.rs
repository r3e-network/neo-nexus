//! Explicit resolution of generated configuration conflicts.
use super::super::{html, WebState};
use crate::{
    config::{config_conflict, list_config_conflicts, resolve_config_conflict},
    types::NodeConfig,
};
use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

fn paths(state: &WebState, node: &NodeConfig) -> anyhow::Result<Vec<std::path::PathBuf>> {
    Ok(
        list_config_conflicts(&state.workspace_child_dir("nodes").join(&node.id))?
            .into_iter()
            .map(|conflict| conflict.path)
            .collect(),
    )
}

pub(super) fn render(state: &WebState, node: &NodeConfig) -> String {
    let result = (|| -> anyhow::Result<String> {
        let mut body = String::new();
        for (index, path) in paths(state, node)?.into_iter().enumerate() {
            let Some(conflict) = config_conflict(&path)? else {
                continue;
            };
            body.push_str(&format!("<h3>{}</h3>{}", html::escape(&node.name), html::note(&format!(
                "Configuration conflict: {} (runtime {} → {}). Compare this local file with candidate {}. Keep local accepts the current file for this version; use generated first saves a private backup. Retry Start or Upgrade after resolving all files.",
                path.display(), conflict.from_version, conflict.to_version, conflict.candidate_path.display()))));
            for (choice, label) in [
                ("local", "Keep local"),
                ("generated", "Back up local and use generated"),
            ] {
                body.push_str(&html::control_form(
                    &format!("/config/{}/resolve", node.id),
                    &[
                        ("file", &index.to_string()),
                        ("token", &conflict.token),
                        ("choice", choice),
                    ],
                    label,
                ));
            }
        }
        Ok(body)
    })();
    result.unwrap_or_else(|error| html::note(&format!("configuration review unavailable: {error}")))
}

#[derive(Deserialize)]
pub struct Resolution {
    file: usize,
    token: String,
    choice: String,
}

pub async fn resolve(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(input): Form<Resolution>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let supervisor = state.supervisor();
        let node = state
            .repository
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == id)
            .ok_or_else(|| anyhow::anyhow!("node not found"))?;
        if node.status.is_running() || node.pid.is_some() || supervisor.is_managing(&id) {
            anyhow::bail!("stop the node before resolving configuration conflicts");
        }
        let keep_local = match input.choice.as_str() {
            "local" => true,
            "generated" => false,
            _ => anyhow::bail!("unknown resolution"),
        };
        let files = paths(&state, &node)?;
        let path = files
            .get(input.file)
            .ok_or_else(|| anyhow::anyhow!("config file not found"))?;
        let backup = resolve_config_conflict(path, &input.token, keep_local)?;
        Ok(match backup {
            Some(path) => format!(
                "configuration resolved; previous file backed up at {}",
                path.display()
            ),
            None => "local configuration accepted for the reviewed runtime version".into(),
        })
    })();
    let message = outcome.unwrap_or_else(|error| format!("configuration not changed: {error}"));
    Redirect::to(&format!(
        "/config?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}

pub async fn review(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let _supervisor = state.supervisor();
        let node = state
            .repository
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == id)
            .ok_or_else(|| anyhow::anyhow!("node not found"))?;
        let path = crate::config::ConfigExporter::managed_target_path(
            state.workspace_child_dir("nodes").join(&id),
            &node,
        );
        let plugins = state.repository.list_plugin_states(&id)?;
        let context = crate::node_lifecycle::generation_context_for_node(&state.repository, &node);
        let count =
            crate::config::ConfigExporter::review_node_config(&path, &node, &plugins, &context)?;
        Ok(format!(
            "{} configuration conflict(s) staged for {}; active files preserved",
            count, node.name
        ))
    })();
    let message = outcome.unwrap_or_else(|error| format!("review failed: {error}"));
    Redirect::to(&format!(
        "/config?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}

#[cfg(test)]
#[path = "../../../tests/unit/web/config_conflicts.rs"]
mod tests;
