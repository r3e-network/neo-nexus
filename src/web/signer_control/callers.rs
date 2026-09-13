//! Custody caller operations: creation, rotation, disabling, deletion, and workload assertions.

use axum::{
    extract::{Form, Path, RawForm, State},
    response::Response,
};

use crate::{
    signer_client::WorkloadCallerRequest,
    web::{pages::signer, WebState},
};

use super::{
    forms::{amount, entries, flag, grant, workload_caller_form, CallerForm, SwitchForm},
    manage, respond,
};

pub async fn create_caller(
    State(state): State<WebState>,
    Form(form): Form<CallerForm>,
) -> Response {
    let outcome = state
        .custody()
        .ask(move |admin| {
            let grant = grant(&form.grant, &form.keys)?;
            let created = admin
                .client()
                .create_caller(
                    &admin.credentials()?,
                    &form.label,
                    &grant,
                    std::slice::from_ref(&form.capability),
                    &entries(&form.origins),
                )?
                .into_parts()?;
            Ok(created)
        })
        .await;
    match outcome {
        Ok(created) => {
            signer::render(
                &state,
                &format!("caller {} created", created.caller.label),
                Some(&created.token),
            )
            .await
        }
        Err(error) => signer::render(&state, &format!("not saved: {error}"), None).await,
    }
}

pub async fn create_workload_caller(
    State(state): State<WebState>,
    RawForm(body): RawForm,
) -> Response {
    let path = "/signer".to_string();
    let form = match workload_caller_form(&body) {
        Ok(form) => form,
        Err(error) => return respond(&path, Err(error)),
    };
    manage(&state, path, move |admin| {
        let request = WorkloadCallerRequest {
            label: form.label.trim().to_string(),
            key_grant: grant(&form.grant, &form.keys)?,
            capabilities: vec![form.capability.trim().to_string()],
            allowed_origins: entries(&form.origins),
            workload_public_key: form.workload_public_key.trim().to_string(),
            workload_subject: amount(&form.workload_subject).map(str::to_string),
        };
        let created = admin
            .client()
            .create_workload_caller(&admin.credentials()?, &request)?
            .into_parts()?;
        Ok(format!(
            "workload caller {} registered — its Ed25519 private key remains in the workload",
            created.caller.label
        ))
    })
    .await
}

pub async fn rotate_caller(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let outcome = state
        .custody()
        .ask(move |admin| {
            let rotated = admin
                .client()
                .rotate_caller_token(&admin.credentials()?, &id)?
                .into_parts()?;
            Ok(rotated)
        })
        .await;
    match outcome {
        // The old token stops working at this moment, which is the point the
        // operator has to be told plainly: a rotation that left two live tokens
        // would not be a revocation.
        Ok(rotated) => {
            signer::render(
                &state,
                &format!(
                    "caller {} rotated — its previous token no longer works",
                    rotated.caller_id
                ),
                Some(&rotated.token),
            )
            .await
        }
        Err(error) => signer::render(&state, &format!("not rotated: {error}"), None).await,
    }
}

pub async fn set_caller_state(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<SwitchForm>,
) -> Response {
    let path = "/signer".to_string();
    manage(&state, path, move |admin| {
        let disabled = flag(&form.disabled, "caller state")?;
        let caller = admin
            .client()
            .set_caller_disabled(&admin.credentials()?, &id, disabled)?
            .into_parts()?;
        Ok(format!(
            "{} is now {}",
            caller.label,
            if disabled { "disabled" } else { "enabled" }
        ))
    })
    .await
}

pub async fn delete_caller(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let path = "/signer".to_string();
    manage(&state, path, move |admin| {
        admin
            .client()
            .delete_caller(&admin.credentials()?, &id)?
            .into_parts()?;
        Ok(format!("caller {id} deleted"))
    })
    .await
}
