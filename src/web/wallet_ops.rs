//! Wallet profile management operations for the web UI.
//! This module implements user-facing handlers for importing and deleting wallet profiles.
//! The handlers reuse existing repository methods and core wallet validation logic.

use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use tokio::task::spawn_blocking;

use crate::core::operations::{EventKind, EventSeverity, NewRuntimeEvent};
use crate::web::{
    form_state::{FieldErrors, GENERAL},
    html,
    pages::wallets::{self, WalletImportDraft, WALLET_PATH},
    WebState,
};

/// What the import refused, and which control to point at.
///
/// A wallet file that will not validate is refused *about the file* — the path,
/// the bytes, the accounts inside it — so the path is the control to blame.
/// Failing to store a profile that did validate is a workspace problem and
/// belongs to no field at all.
struct Refusal {
    field: &'static str,
    message: String,
}

impl Refusal {
    fn at(field: &'static str, error: impl std::fmt::Display) -> Self {
        Self {
            field,
            message: error.to_string(),
        }
    }
}

/// POST /wallets/import — Import a new wallet profile from a local server-side file path.
/// This handler runs synchronously because workspace queries + validation don't block.
pub async fn import_wallet_profile(
    State(state): State<WebState>,
    Form(draft): Form<WalletImportDraft>,
) -> impl IntoResponse {
    // Validate inputs
    if draft.file_path.trim().is_empty() {
        // The page the operator came from, with the problem stated on it. A
        // bare `400` would take the navigation and the form away with it.
        return redraw(
            &state,
            &draft,
            WALLET_PATH,
            "Give the path of the wallet file to import.",
        );
    }

    // Run crypto ops in blocking task to avoid blocking the async runtime
    let input = draft.clone();
    // The blocking task needs its own handle: the request's is still needed
    // afterwards, to draw the page again if the import is refused.
    let state_for_task = state.clone();
    let outcome = spawn_blocking(move || {
        use std::path::Path;

        let path = Path::new(input.file_path.trim());

        // Security check: ensure the resolved wallet path is under the workspace
        // data directory. Canonicalize both sides so the comparison holds on
        // Windows, where `canonicalize` yields an extended-length prefix.
        let canonical_wallet = path.canonicalize().map_err(|_| {
            Refusal::at(
                WALLET_PATH,
                format!("wallet file not found: {}", input.file_path),
            )
        })?;
        let canonical_root = state_for_task
            .data_dir
            .canonicalize()
            .unwrap_or_else(|_| state_for_task.data_dir.clone());

        if !canonical_wallet.starts_with(&canonical_root) {
            return Err(Refusal::at(
                WALLET_PATH,
                format!(
                    "wallet path must be within the workspace directory: {}",
                    input.file_path
                ),
            ));
        }

        // Use NeoWalletValidator from wallet module
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| {
                Refusal::at(
                    GENERAL,
                    anyhow::anyhow!("system clock is before Unix epoch: {error}"),
                )
            })?
            .as_secs();
        let profile = crate::wallet::NeoWalletValidator::profile_from_path(
            path,
            input.id.trim(),
            input.label.trim(),
            now,
        )
        .map_err(|error| Refusal::at(WALLET_PATH, error))?;

        // Persist via upsert_neo_wallet_profile
        state_for_task
            .commands
            .upsert_neo_wallet_profile(&profile)
            .map_err(|error| Refusal::at(GENERAL, error))?;

        // Record event immediately, before leaving the closure
        let _ = state_for_task.commands.record_event(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::NeoWalletProfileImported,
            severity: EventSeverity::Info,
            message: format!(
                "wallet '{}' imported ({})",
                profile.label, profile.primary_address
            ),
        });

        Ok(profile)
    })
    .await;

    match outcome {
        Ok(Ok(profile)) => {
            // Event was already recorded inside spawn_blocking
            let flash =
                html::Flash::ok(format!("wallet '{}' imported successfully", profile.label));
            Redirect::to(&flash.redirect("/wallets")).into_response()
        }
        Ok(Err(refusal)) => redraw(&state, &draft, refusal.field, &refusal.message),
        Err(joined) => redraw(
            &state,
            &draft,
            GENERAL,
            &format!("the import task did not finish: {joined}"),
        ),
    }
}

/// The wallets page again, with the form as posted and the reason it was
/// refused.
fn redraw(
    state: &WebState,
    draft: &WalletImportDraft,
    field: &'static str,
    message: &str,
) -> Response {
    let mut errors = FieldErrors::new();
    errors.insert(field, message.to_string());
    match wallets::render_with_import_draft(&state.workspace, draft, &errors) {
        Ok(body) => Html(html::layout(
            "Wallets",
            "wallets",
            html::Flash::default(),
            &body,
        ))
        .into_response(),
        Err(error) => {
            let flash = html::Flash::danger(format!(
                "not imported: {message} — and the page could not be redrawn ({error}), so the \
                 form was lost"
            ));
            Redirect::to(&flash.redirect("/wallets")).into_response()
        }
    }
}

/// GET /wallets/{id}/delete — Show confirmation form for wallet deletion.
pub async fn show_delete_form(Path(id): Path<String>) -> Response {
    let body = format!(
        r#"<h1>Delete Wallet Profile</h1>
<p>You are about to delete wallet profile <strong>{name}</strong>.</p>
<p>This action cannot be undone.</p>
<form method="POST" action="/wallets/{id}/delete">
<div class="form-actions">
<button class="danger" type="submit">Confirm Delete</button>
<a class="btn" href="/wallets">Cancel</a>
</div>
</form>"#,
        name = html::escape(&id),
        id = html::escape(&id)
    );

    html::layout(
        "Delete Wallet",
        "wallets",
        html::Flash::danger("This will permanently remove the wallet profile from this workspace."),
        &body,
    )
    .into_response()
}

/// POST /wallets/{id}/delete — Execute wallet profile deletion and record NeoWalletProfileDeleted.
pub async fn delete_wallet_profile(
    State(state): State<WebState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // A delete against a wallet that is not there is a 404, not a redirect:
    // there is no listing row the operator could have clicked from, so the
    // request names something the workspace never held.
    let exists = state
        .workspace
        .list_neo_wallet_profiles()
        .map(|profiles| profiles.iter().any(|profile| profile.id == id))
        .unwrap_or(false);
    if !exists {
        return (StatusCode::NOT_FOUND, "wallet profile not found").into_response();
    }

    // Execute synchronous delete (single row, small transaction).
    let flash = match state.commands.delete_neo_wallet_profile(&id) {
        Ok(()) => {
            let message = format!("deleted wallet '{id}'");
            // Record the deletion only once it has actually happened.
            let _ = state.commands.record_event(NewRuntimeEvent {
                node_id: None,
                node_name: None,
                kind: EventKind::NeoWalletProfileDeleted,
                severity: EventSeverity::Info,
                message: message.clone(),
            });
            html::Flash::ok(message)
        }
        Err(error) => html::Flash::danger(format!("failed to delete wallet: {error}")),
    };

    Redirect::to(&flash.redirect("/wallets")).into_response()
}
