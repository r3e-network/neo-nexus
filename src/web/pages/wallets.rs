//! Wallets: the Neo wallet profiles this workspace has inspected. The repository
//! stores metadata only — address, account counts, and the digest of the wallet
//! file — never private keys, passwords, or wallet bytes, and this page shows
//! exactly that stored metadata so nothing sensitive is pulled toward a browser.

use axum::{
    extract::{Query, RawQuery, State},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

use crate::core::security::{filter_neo_wallet_profiles, NeoWalletProfile, NeoWalletProfileFilter};

use super::super::{
    form_state::{self, FieldErrors, FormDraft},
    html, time, WebState,
};

/// The three fields the import form carries, named once so the page that draws
/// them and the handler that blames one cannot disagree.
pub const WALLET_PATH: &str = "file_path";
pub const WALLET_ID: &str = "id";
pub const WALLET_LABEL: &str = "label";

/// The import form as posted, every value still the text the browser sent.
///
/// A wallet that fails to validate used to cost all three fields — and the one
/// that takes longest to type is the path, which is exactly the field the
/// failure is usually about.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct WalletImportDraft {
    #[serde(default)]
    pub file_path: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub label: String,
}

impl FormDraft for WalletImportDraft {
    fn value(&self, field: &str) -> &str {
        match field {
            WALLET_PATH => &self.file_path,
            WALLET_ID => &self.id,
            WALLET_LABEL => &self.label,
            _ => "",
        }
    }

    fn checked(&self, _field: &str) -> bool {
        false
    }
}

#[derive(Default, Deserialize)]
pub struct WalletQuery {
    #[serde(default)]
    used: String,
    #[serde(default)]
    q: String,
}

pub async fn wallets(
    State(state): State<WebState>,
    RawQuery(flash): RawQuery,
    Query(params): Query<WalletQuery>,
) -> Response {
    let body = match state.workspace.list_neo_wallet_profiles() {
        Ok(profiles) => {
            let filter = NeoWalletProfileFilter::new(tri_state(&params.used), params.q.trim());
            render_body(
                &profiles,
                &filter_neo_wallet_profiles(&profiles, &filter),
                &params,
                None,
            )
        }
        Err(error) => html::note(&format!("failed to load wallet profiles: {error}")),
    };
    Html(html::layout(
        "Wallets",
        "wallets",
        html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

/// The page again, with the import form holding what was posted and the reason
/// it was refused.
///
/// The profile table above is still what the workspace holds: nothing was
/// stored, and the page says so rather than looking like a save that quietly did
/// not happen.
pub(crate) fn render_with_import_draft(
    workspace: &crate::core::workspace_queries::WorkspaceQueries,
    draft: &WalletImportDraft,
    errors: &FieldErrors,
) -> anyhow::Result<String> {
    let profiles = workspace.list_neo_wallet_profiles()?;
    // A redraw has no query string, so the table is unfiltered. Narrowing is a
    // view concern; losing it costs a click, while losing the form costs three
    // fields.
    let params = WalletQuery::default();
    let filter = NeoWalletProfileFilter::new(tri_state(&params.used), params.q.trim());
    Ok(render_body(
        &profiles,
        &filter_neo_wallet_profiles(&profiles, &filter),
        &params,
        Some((draft, errors)),
    ))
}

fn tri_state(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" => Some(true),
        "no" | "false" => Some(false),
        _ => None,
    }
}

fn render_body(
    all: &[NeoWalletProfile],
    visible: &[NeoWalletProfile],
    params: &WalletQuery,
    import: Option<(&WalletImportDraft, &FieldErrors)>,
) -> String {
    format!(
        r#"<h1>Wallets</h1>
{import_form}
{tiles}
{filters}
{table}
{privacy}"#,
        import_form = render_import_form(import),
        tiles = html::cards(&[
            ("Profiles", all.len().to_string()),
            (
                "In use",
                all.iter()
                    .filter(|profile| profile.last_used_at_unix.is_some())
                    .count()
                    .to_string(),
            ),
            (
                "Accounts",
                all.iter()
                    .map(|profile| profile.account_count)
                    .sum::<usize>()
                    .to_string(),
            ),
            (
                "Encrypted",
                all.iter()
                    .map(|profile| profile.encrypted_account_count)
                    .sum::<usize>()
                    .to_string(),
            ),
        ]),
        filters = html::typed_filter_form(
            "/wallets",
            &[],
            &[
                html::FilterControl::Select {
                    label: "Usage",
                    name: "used",
                    selected: &params.used,
                    options: &[("", "All wallets"), ("yes", "In use"), ("no", "Not in use")],
                },
                html::FilterControl::Search {
                    label: "Search",
                    name: "q",
                    value: &params.q,
                    placeholder: "Wallet, node, or path",
                },
            ],
        ),
        table = wallet_table(visible),
        privacy = html::note(
            "Only validation metadata is stored: no private keys, passwords, or wallet bytes ever reach this page.",
        ),
    )
}

fn render_import_form(import: Option<(&WalletImportDraft, &FieldErrors)>) -> String {
    let (values, errors) = match import {
        Some((values, errors)) => (Some(values), Some(errors)),
        None => (None, None),
    };
    let value = |field: &str| values.map_or("", |draft| draft.value(field));
    let error = |field: &str| errors.and_then(|errors| form_state::error_for(errors, field));
    let summary = match errors {
        Some(errors) if !errors.is_empty() => format!(
            "{}{}",
            form_state::banner(errors),
            html::note(
                "Nothing was imported. The profiles above are unchanged; the form holds what \
                 you typed and what it would not accept."
            ),
        ),
        _ => String::new(),
    };
    // The console is a dark surface. A hand-written light form on it is
    // unreadable, and the inline colours here were not the palette the rest
    // of the product uses.
    format!(
        r#"<form method="POST" action="/wallets/import" class="panel" style="max-width: 420px; margin-bottom: 24px; padding: 16px;">
          {summary}
          <div style="margin-bottom: 12px;">{path_field}</div>
          <div style="margin-bottom: 12px;">{id_field}</div>
          <div style="margin-bottom: 12px;">{label_field}</div>
          <button type="submit" class="btn primary">Import Wallet</button>
        </form>"#,
        path_field = html::TextField {
            label: "Wallet File Path",
            name: WALLET_PATH,
            value: value(WALLET_PATH),
            placeholder: Some("/path/to/wallet.json"),
            monospace: true,
            required: true,
            full_width: true,
            error: error(WALLET_PATH),
            ..html::TextField::default()
        }
        .render(),
        id_field = html::TextField {
            label: "ID (optional)",
            name: WALLET_ID,
            value: value(WALLET_ID),
            placeholder: Some("auto-generated if empty"),
            full_width: true,
            error: error(WALLET_ID),
            ..html::TextField::default()
        }
        .render(),
        label_field = html::TextField {
            label: "Label (optional)",
            name: WALLET_LABEL,
            value: value(WALLET_LABEL),
            placeholder: Some("Descriptive name"),
            full_width: true,
            error: error(WALLET_LABEL),
            ..html::TextField::default()
        }
        .render(),
    )
}

fn wallet_table(profiles: &[NeoWalletProfile]) -> String {
    if profiles.is_empty() {
        return html::empty_state(
            "No wallet profiles",
            "No wallet has been validated in this workspace yet. Import one to see the metadata \
             this workspace keeps about it.",
            r#"<a class="btn" href="/wallets">Import a wallet</a>"#,
        );
    }
    let rows = profiles
        .iter()
        .map(|profile| {
            html::row(&[
                html::cell(&profile.label),
                html::cell(&profile.primary_address),
                html::cell(&profile.source_path),
                html::cell(profile.wallet_version.as_deref().unwrap_or("unknown")),
                html::cell(&profile.account_count.to_string()),
                html::cell(&profile.encrypted_account_count.to_string()),
                html::cell(&profile.watch_only_account_count.to_string()),
                html::cell(&profile.contract_public_keys.len().to_string()),
                html::cell(&profile.wallet_sha256.chars().take(12).collect::<String>()),
                html::raw_cell(&time::time_cell(Some(profile.validated_at_unix))),
                html::raw_cell(&format!(
                    r#"<a class="btn small danger" href="/wallets/{}/delete">Delete</a>"#,
                    html::escape(&profile.id)
                )),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Label",
            "Primary address",
            "Source",
            "Version",
            "Accounts",
            "Encrypted",
            "Watch-only",
            "Keys",
            "File SHA-256",
            "Validated",
            "Delete",
        ],
        &rows,
    )
}
