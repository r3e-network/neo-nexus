//! Custody key operations: generation, disabling, deletion, and policy definition.

use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    signer_client::Outcome,
    web::{
        form_state::{FieldErrors, GENERAL},
        html,
        pages::signer,
        WebState,
    },
};

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

/// Save a boundary, and keep the operator's text when it does not save.
///
/// Twenty-nine fields is too many to retype because one of them was refused. So a
/// refusal comes back as the same page with the posted text still in it and the
/// sentence beside the field it belongs to. There are three ways that happens,
/// and they are not the same failure:
///
/// - this console could not read the form, so it knows which field is wrong;
/// - the service answered with a refusal of its own, which belongs to no single
///   field and is shown in the service's words;
/// - the service could not be reached, so there is no answer to show beside
///   anything — it goes in the same general place, and if the page cannot be
///   redrawn either, the redirect carries the message instead.
///
/// What all three have in common is that the boundary was *not* stored. A save
/// that succeeded still redirects: the page the browser lands on reads the
/// boundary back from the service, and a 200 here would claim a state this
/// console only knows the service's answer to.
pub async fn save_policy(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<PolicyForm>,
) -> Response {
    let policy = match build_policy(&form) {
        Ok(policy) => policy,
        Err(refusal) => {
            return refused(&state, &id, &form, refusal.field, &refusal.message).await;
        }
    };
    let asked = id.clone();
    let answered = state
        .custody()
        .ask(move |admin| {
            admin
                .client()
                .save_policy(&admin.credentials()?, &asked, &policy)
        })
        .await;
    match answered {
        Ok(Outcome::Allowed(stored)) => {
            // The service's own advice about the shape it just stored, counted
            // rather than spelled out: the page this redirect lands on lists
            // them in full.
            let warning = if stored.problems.is_empty() {
                String::new()
            } else {
                format!(
                    " — stored, and {} of its shapes look weaker than intended",
                    stored.problems.len()
                )
            };
            let policy = stored.policy;
            let flash = html::Flash::ok(format!(
                "boundary saved for {id}{warning}: consensus {}, raw {}, transfers {}, calls {}, \
                 global scope {}",
                word(policy.allow_consensus),
                word(policy.allow_raw),
                word(policy.allow_transfer),
                word(policy.allow_contract_call),
                word(policy.allow_global_scope),
            ));
            Redirect::to(&flash.redirect(&key_path(&id))).into_response()
        }
        Ok(Outcome::Refused(refusal)) => {
            refused(&state, &id, &form, GENERAL, &refusal.summary()).await
        }
        Err(error) => refused(&state, &id, &form, GENERAL, &format!("{error}")).await,
    }
}

/// Put a refusal beside the field it belongs to, on a page that still has the
/// operator's text in it.
///
/// Drawing that page needs the key, a slice of its audit and the caller list, so
/// redrawing it is another conversation with the service. If that conversation
/// fails — the service went down between the save and the redraw, most likely —
/// the input cannot be kept, and the redirect says so rather than pretending the
/// form came back.
async fn refused(
    state: &WebState,
    id: &str,
    form: &PolicyForm,
    field: &'static str,
    message: &str,
) -> Response {
    let mut errors = FieldErrors::new();
    errors.insert(field, message.to_string());
    match signer::render_with_policy_draft(state, id, form, &errors).await {
        Ok(response) => response,
        Err(error) => {
            let flash = html::Flash::danger(format!(
                "not saved: {message} — and the page could not be redrawn ({error}), so the form \
                 was lost"
            ));
            Redirect::to(&flash.redirect(&key_path(id))).into_response()
        }
    }
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
