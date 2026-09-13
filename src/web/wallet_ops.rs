//! Wallet profile management operations for the web UI.
//! This module implements user-facing handlers for importing and deleting wallet profiles.
//! The handlers reuse existing repository methods and core wallet validation logic.

use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use tokio::task::spawn_blocking;

use crate::core::operations::{EventKind, EventSeverity, NewRuntimeEvent};
use crate::web::{html, WebState};
use anyhow::Context;

/// Form for importing a wallet profile (accepts path string + optional metadata).
#[derive(Debug, Clone, Deserialize)]
pub struct WalletImportForm {
    pub file_path: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}

/// POST /wallets/import — Import a new wallet profile from a local server-side file path.
/// This handler runs synchronously because workspace queries + validation don't block.
pub async fn import_wallet_profile(
    State(state): State<WebState>,
    Form(form): Form<WalletImportForm>,
) -> impl IntoResponse {
    // Validate inputs
    if form.file_path.is_empty() {
        return (StatusCode::BAD_REQUEST, "file_path is required").into_response();
    }

    // Run crypto ops in blocking task to avoid blocking the async runtime
    let outcome = spawn_blocking(move || {
        use std::path::Path;

        let path = Path::new(&form.file_path);

        // Security check: ensure the resolved wallet path is under the workspace
        // data directory. Canonicalize both sides so the comparison holds on
        // Windows, where `canonicalize` yields an extended-length prefix.
        let canonical_wallet = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return Err(anyhow::anyhow!("wallet file not found: {}", form.file_path)),
        };
        let canonical_root = state
            .data_dir
            .canonicalize()
            .unwrap_or_else(|_| state.data_dir.clone());

        if !canonical_wallet.starts_with(&canonical_root) {
            return Err(anyhow::anyhow!(
                "wallet path must be within the workspace directory: {}",
                form.file_path
            ));
        }

        // Get input values
        let id_input = form.id.as_deref().unwrap_or("");
        let label_input = form.label.as_deref().unwrap_or("");

        // Use NeoWalletValidator from wallet module
        let profile = crate::wallet::NeoWalletValidator::profile_from_path(
            path,
            id_input,
            label_input,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .context("system clock is before Unix epoch")?
                .as_secs(),
        )?;

        // Persist via upsert_neo_wallet_profile
        state.commands.upsert_neo_wallet_profile(&profile)?;

        // Record event immediately, before leaving the closure
        let _ = state.commands.record_event(NewRuntimeEvent {
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

    let message = match outcome {
        Ok(Ok(profile)) => {
            // Event was already recorded inside spawn_blocking
            format!("wallet '{}' imported successfully", profile.label)
        }
        Ok(Err(error)) => error.to_string(),
        Err(_) => "import operation failed".to_string(),
    };

    Redirect::to(&format!(
        "/wallets?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}

/// GET /wallets/{id}/delete — Show confirmation form for wallet deletion.
pub async fn show_delete_form(Path(id): Path<String>) -> Response {
    let body = format!(
        r#"<h1>Delete Wallet Profile</h1>
<p>You are about to delete wallet profile <strong>{}</strong>.</p>
<p>This action cannot be undone.</p>
<form method="POST" action="/wallets/{}/delete">
  <button type="submit" style="background-color: #d9534f; color: white; padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer;">Confirm Delete</button>
  <a href="/wallets"><button type="button" style="background-color: #5bc0de; color: white; padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer;">Cancel</button></a>
</form>"#,
        html::escape(&id),
        html::escape(&id)
    );

    html::layout(
        "Delete Wallet",
        "wallets",
        &html::note(
            "Warning: This will permanently remove the wallet profile from this workspace.",
        ),
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
    let message = match state.commands.delete_neo_wallet_profile(&id) {
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
            message
        }
        Err(error) => format!("failed to delete wallet: {error}"),
    };

    Redirect::to(&format!(
        "/wallets?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}
