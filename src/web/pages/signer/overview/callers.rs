use crate::{
    signer_client::{Caller, Grant, KeyPublic},
    web::html,
};

use super::panels::{field_with_id, state_badge};

pub fn caller_table(callers: &[Caller]) -> String {
    if callers.is_empty() {
        return html::empty_state(
            "No callers registered",
            "A key with no caller is held closed. Add only the workload identities that need to ask custody.",
            "",
        );
    }
    let rows = callers
        .iter()
        .map(|caller| {
            let path = html::urlencoding_lite(&caller.id);
            let rotate = match caller.auth_mode.as_deref() {
                None | Some("bearer") => format!(
                    r#"<a class="btn small danger" href="/signer/callers/{path}/rotate">Rotate token</a>"#
                ),
                Some(mode) => format!(
                    r#"<span class="muted">{} identity: no bearer token to rotate</span>"#,
                    html::escape(mode)
                ),
            };
            html::row(&[
                html::cell(&caller.label),
                html::cell(&caller_identity(caller)),
                html::cell(&caller.capabilities.join(" + ")),
                html::cell(&grant_summary(&caller.key_grant)),
                html::cell(&origins_summary(&caller.allowed_origins)),
                html::raw_cell(&state_badge(!caller.disabled)),
                html::raw_cell(&format!(
                    r#"<div class="row-actions">{}{}<a class="btn small danger" href="/signer/callers/{path}/delete">Delete</a></div>"#,
                    html::control_form(
                        &format!("/signer/callers/{path}/state"),
                        &[("disabled", if caller.disabled { "false" } else { "true" })],
                        if caller.disabled { "Enable" } else { "Disable" }
                    ),
                    rotate,
                )),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Label", "Identity", "May", "Keys", "Origins", "Status", "Actions",
        ],
        &rows,
    )
}

pub fn caller_identity(caller: &Caller) -> String {
    match caller.auth_mode.as_deref() {
        Some("workload-ed25519") => caller.workload_subject.as_deref().map_or_else(
            || "workload-ed25519".to_string(),
            |subject| format!("workload-ed25519 · {subject}"),
        ),
        Some(mode) => mode.to_string(),
        None => "bearer (legacy signer)".to_string(),
    }
}

pub fn grant_summary(grant: &Grant) -> String {
    if grant.is_any() {
        "any key".to_string()
    } else {
        match grant.key_ids.len() {
            0 => "no keys".to_string(),
            count => format!("{count} keys"),
        }
    }
}

pub fn origins_summary(origins: &[String]) -> String {
    if origins.is_empty() {
        "server-to-server (no browser origin accepted)".to_string()
    } else {
        origins.join(", ")
    }
}

pub fn key_options(keys: &[KeyPublic]) -> String {
    keys.iter()
        .map(|key| {
            format!(
                r#"<option value="{}">{}</option>"#,
                html::escape(&key.key_id),
                html::escape(&format!("{} — {}", key.label, key.address))
            )
        })
        .collect()
}

pub fn new_caller_form(keys: &[KeyPublic]) -> String {
    format!(
        r#"<div class="panel"><h3>Add a bearer caller</h3><p class="muted">The token is shown once and is never stored in recoverable form.</p>
<form method="post" action="/signer/callers"><div class="grid">{label}
<label class="field"><span>May</span><select name="capability"><option value="sign">Sign — ask a granted key</option><option value="admin">Admin — configure custody</option></select></label>
<label class="field"><span>Keys</span><select name="grant"><option value="any">Any key</option><option value="only">Only chosen keys</option></select></label>
<label class="field span-all"><span>Granted keys</span><select name="keys" multiple size="4">{options}</select><span class="help">Choosing none under Only chosen keys grants none.</span></label>
<label class="field span-all"><span>Allowed origins</span><input name="origins" class="mono" placeholder="https://relayer.example.com"><span class="help">Comma separated; blank is server-to-server only.</span></label></div>
<button type="submit">Create caller</button></form></div>"#,
        label = field_with_id("bearer-caller-label", "Label", "label", "", None),
        options = key_options(keys),
    )
}

pub fn new_workload_caller_form(keys: &[KeyPublic]) -> String {
    format!(
        r#"<div class="panel"><h3>Add a workload identity</h3><p class="muted">Only its Ed25519 public key enters NeoNexus; the private key remains in the workload.</p>
<form method="post" action="/signer/callers/workload"><div class="grid">{label}
<label class="field"><span>May</span><select name="capability"><option value="sign">Sign parsed transactions</option><option value="raw_sign">Sign raw bytes</option><option value="admin">Administer custody</option></select></label>
<label class="field"><span>Keys</span><select name="grant"><option value="only">Only chosen keys</option><option value="any">Any key</option></select></label>
<label class="field span-all"><span>Granted keys</span><select name="keys" multiple size="4">{options}</select></label>
{public_key}{subject}
<label class="field span-all"><span>Allowed origins</span><input name="origins" class="mono" placeholder="leave blank for a non-browser workload"></label></div>
<button type="submit">Register workload</button></form></div>"#,
        label = field_with_id("workload-caller-label", "Label", "label", "", None),
        options = key_options(keys),
        public_key = field_input(
            "Ed25519 public key",
            "workload_public_key",
            "",
            Some("Exactly 32 bytes as 64 hexadecimal characters.")
        ),
        subject = field_input(
            "Workload subject",
            "workload_subject",
            "",
            Some("Optional stable deployment identity.")
        ),
    )
}

fn field_input(label: &str, name: &str, value: &str, help: Option<&str>) -> String {
    html::TextField {
        label,
        name,
        value,
        help,
        ..Default::default()
    }
    .render()
}

pub fn api_reference() -> String {
    r#"<details class="panel"><summary>Caller API reference</summary><p class="muted">Caller credentials, origins, grants and policy are checked and audited by the custody service.</p><pre>POST /signer/api/v1/sign/transaction
POST /signer/api/v1/sign/consensus
POST /signer/api/v1/sign/eip191-fulfillment
POST /signer/api/v1/sign/raw
GET  /signer/api/v1/keys/{key_id}

Authorization: Bearer &lt;caller token&gt;</pre></details>"#
        .to_string()
}
