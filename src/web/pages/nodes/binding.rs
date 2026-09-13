//! Node-to-signer routing configuration and persistence.

use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    core::{
        node::NodeConfig,
        operations::{EventKind, EventSeverity, NewRuntimeEvent},
    },
    signing::SignerKeyRef,
    web::{html, WebState},
};

#[derive(Default, serde::Deserialize)]
#[serde(default)]
pub struct SignerBindingForm {
    pub backend_id: String,
    pub key_id: String,
}

/// Persist one complete signer route for a stopped node. A blank pair clears
/// the route (and therefore disables signing-duty launches); a partial pair is
/// always an error.
pub async fn save_signer_binding(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(input): Form<SignerBindingForm>,
) -> Response {
    let backend_id = input.backend_id.trim();
    let key_id = input.key_id.trim();
    let result = (|| -> anyhow::Result<String> {
        let node = state
            .workspace
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == id)
            .ok_or_else(|| anyhow::anyhow!("node {id} was not found"))?;
        let key = match (backend_id.is_empty(), key_id.is_empty()) {
            (true, true) => None,
            (false, false) => {
                let key = SignerKeyRef::new(backend_id, key_id)?;
                let backend = state.custody().registry().backend(&key.backend_id)?;
                // A process-local wallet has exactly one key. Reject a typo at
                // save time; service keys are authoritatively checked by the
                // service on the first dispatch.
                if let Some(local) = backend.local_wallet_signer() {
                    let actual = local.key_info();
                    if actual.key_id != key.key_id {
                        anyhow::bail!(
                            "local wallet profile {} owns key {}, not {}",
                            key.backend_id,
                            actual.key_id,
                            key.key_id
                        );
                    }
                }
                if let Some(local) = backend.local_signer_config() {
                    if local.public_key() != key.key_id {
                        anyhow::bail!(
                            "local signer profile {} owns public key {}, not {}",
                            key.backend_id,
                            local.public_key(),
                            key.key_id
                        );
                    }
                }
                Some(key)
            }
            _ => anyhow::bail!("signer backend and key id must be set or cleared together"),
        };
        if let Some(ref k) = key {
            let all_bindings = state.workspace.list_all_signer_bindings()?;
            if let Err(violation) =
                crate::signing::check_signer_binding_allowed(&node.id, k, &all_bindings)
            {
                anyhow::bail!("{violation}");
            }
        }
        state.commands.set_node_signer_key(&node.id, key.as_ref())?;
        let message = key.as_ref().map_or_else(
            || format!("{} signer binding cleared", node.name),
            |key| {
                format!(
                    "{} signer bound to {} / {}",
                    node.name, key.backend_id, key.key_id
                )
            },
        );
        let _ = state.commands.record_event(NewRuntimeEvent {
            node_id: Some(node.id),
            node_name: Some(node.name),
            kind: EventKind::NodeSignerBound,
            severity: EventSeverity::Info,
            message: message.clone(),
        });
        Ok(message)
    })();
    let message = result.unwrap_or_else(|error| format!("signer binding not saved: {error}"));
    Redirect::to(&format!(
        "/nodes/{}?flash={}",
        html::urlencoding_lite(&id),
        html::urlencoding_lite(&message)
    ))
    .into_response()
}

pub fn signer_binding(
    state: &WebState,
    node: &NodeConfig,
    selected: Option<&SignerKeyRef>,
) -> String {
    let profiles = state.custody().profiles().collect::<Vec<_>>();
    let selected_backend = selected.map(|key| key.backend_id.as_str()).unwrap_or("");
    let selected_key = selected.map(|key| key.key_id.as_str()).unwrap_or("");
    let current = selected.map_or_else(
        || "Unbound — signing duties cannot start.".to_string(),
        |key| match state.custody().registry().backend(&key.backend_id) {
            Ok(backend) => format!(
                "{} ({}) · key {}",
                backend.profile().label,
                backend.profile().kind,
                key.key_id
            ),
            Err(_) => format!(
                "Unavailable backend {} · key {} (no fallback)",
                key.backend_id, key.key_id
            ),
        },
    );
    if node.status.is_active() || node.pid.is_some() {
        return format!(
            "<h2>Node signer</h2>{}{}",
            html::note(&current),
            html::note("Stop and settle the node before changing its signer identity.")
        );
    }
    let all_bindings = state.workspace.list_all_signer_bindings().unwrap_or_default();
    let options = std::iter::once(
        r#"<option value="">No signer (signing duties disabled)</option>"#.to_string(),
    )
    .chain(profiles.iter().map(|profile| {
        let is_selected = profile.id == selected_backend;
        let other_owner = all_bindings.iter().find(|(nid, b)| b.backend_id == profile.id && nid != &node.id);
        if let Some((owner_id, _)) = other_owner {
            format!(
                r#"<option value="{}" disabled>🔒 {} · {} (Locked by {})</option>"#,
                html::escape(&profile.id),
                html::escape(&profile.label),
                html::escape(profile.kind.label()),
                html::escape(owner_id),
            )
        } else {
            let chosen = if is_selected { " selected" } else { "" };
            format!(
                r#"<option value="{}"{chosen}>{} · {} ({})</option>"#,
                html::escape(&profile.id),
                html::escape(&profile.label),
                html::escape(profile.kind.label()),
                html::escape(&profile.id),
            )
        }
    }))
    .collect::<String>();
    let availability = if profiles.is_empty() {
        html::note("No signer profile is configured. Configure the signer registry before binding this node.")
    } else {
        String::new()
    };
    format!(
        r#"<h2>Node signer</h2>
{current}
{availability}
<div class="notice" style="border-left: 3px solid #2ec274; background: rgba(46, 194, 116, 0.08); margin-bottom: 16px; padding: 12px;">
    <strong>🔒 IAM Instance Profile Security Boundary</strong>
    <div class="help" style="margin-top: 4px;">Signer identities are leased exclusively to exactly one node instance. Cross-node key usage, usurpation, and concurrent consensus signing are strictly blocked.</div>
</div>
<form method="post" action="/nodes/{id}/signer"><div class="panel"><div class="grid">
<label class="field" for="node-signer-backend"><span>Signer backend</span><select id="node-signer-backend" name="backend_id">{options}</select><span class="help">Exactly one local wallet, local signer, or NeoOS signer profile. Unavailable/locked profiles are disabled.</span></label>
<label class="field" for="node-signer-key"><span>Key id</span><input class="mono" id="node-signer-key" name="key_id" value="{key}"><span class="help">The key owned by that backend; both values form one durable route.</span></label>
</div><div class="form-actions"><button type="submit">Save signer binding</button></div></div></form>"#,
        current = html::note(&current),
        id = html::urlencoding_lite(&node.id),
        key = html::escape(selected_key),
    )
}
