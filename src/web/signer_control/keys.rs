//! Custody key operations: generation, disabling, deletion, and policy definition.

use axum::{
    extract::{Form, Path, State},
    response::Response,
};

use crate::web::{html, WebState};

use super::{
    forms::{build_policy, flag, generate_key_request, NewKeyForm, PolicyForm, SwitchForm},
    manage,
};

pub async fn generate(State(state): State<WebState>, Form(form): Form<NewKeyForm>) -> Response {
    let path = "/signer".to_string();
    manage(&state, path, move |admin| {
        let request = generate_key_request(&form)?;
        let key = admin
            .client()
            .generate_key_request(&admin.credentials()?, &request)?
            .into_parts()?;
        Ok(format!(
            "custody key {} generated — {} (no boundary yet, so it signs nothing)",
            key.label, key.address
        ))
    })
    .await
}

pub async fn set_key_state(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<SwitchForm>,
) -> Response {
    let path = key_path(&id);
    manage(&state, path, move |admin| {
        let disabled = flag(&form.disabled, "signing state")?;
        let key = admin
            .client()
            .set_key_disabled(&admin.credentials()?, &id, disabled)?
            .into_parts()?;
        Ok(format!(
            "{} is now {}",
            key.label,
            if disabled { "disabled" } else { "enabled" }
        ))
    })
    .await
}

pub async fn delete_key(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let path = "/signer".to_string();
    manage(&state, path, move |admin| {
        admin
            .client()
            .delete_key(&admin.credentials()?, &id)?
            .into_parts()?;
        Ok(format!(
            "custody key {id} deleted; its audit history was kept"
        ))
    })
    .await
}

pub async fn save_policy(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<PolicyForm>,
) -> Response {
    let path = key_path(&id);
    manage(&state, path, move |admin| {
        let policy = build_policy(&form)?;
        let stored = admin
            .client()
            .save_policy(&admin.credentials()?, &id, &policy)?
            .into_parts()?;
        // The service's own advice about the shape it just stored, counted rather
        // than spelled out: the page this redirect lands on lists them in full.
        let warning = if stored.problems.is_empty() {
            String::new()
        } else {
            format!(
                " — stored, and {} of its shapes look weaker than intended",
                stored.problems.len()
            )
        };
        let policy = stored.policy;
        Ok(format!(
            "boundary saved for {id}{warning}: consensus {}, raw {}, transfers {}, calls {}, \
             global scope {}",
            word(policy.allow_consensus),
            word(policy.allow_raw),
            word(policy.allow_transfer),
            word(policy.allow_contract_call),
            word(policy.allow_global_scope),
        ))
    })
    .await
}

pub fn key_path(id: &str) -> String {
    format!("/signer/keys/{}", html::urlencoding_lite(id))
}

fn word(flag: bool) -> &'static str {
    if flag {
        "allowed"
    } else {
        "closed"
    }
}
