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

use super::super::{html, time, WebState};

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
    let body = match state.repository.list_neo_wallet_profiles() {
        Ok(profiles) => {
            let filter = NeoWalletProfileFilter::new(tri_state(&params.used), params.q.trim());
            render_body(
                &profiles,
                &filter_neo_wallet_profiles(&profiles, &filter),
                &params,
            )
        }
        Err(error) => html::note(&format!("failed to load wallet profiles: {error}")),
    };
    Html(html::layout(
        "Wallets",
        "wallets",
        &html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
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
) -> String {
    format!(
        r#"<h1>Wallets</h1>
{import_form}
{tiles}
{filters}
{table}
{privacy}"#,
        import_form = render_import_form(),
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

fn render_import_form() -> String {
    r#"<form method="POST" action="/wallets/import" style="max-width: 300px; margin-bottom: 24px; padding: 16px; background-color: #f8f9fa; border-radius: 8px;">          <div style="margin-bottom: 12px;">
            <label for="file_path" style="display: block; margin-bottom: 4px; font-weight: bold;">Wallet File Path:</label>
            <input type="text" id="file_path" name="file_path" required style="width: 100%; padding: 8px; border: 1px solid #ced4da; border-radius: 4px; box-sizing: border-box;" placeholder="/path/to/wallet.json">
          </div>
          <div style="margin-bottom: 12px;">
            <label for="id" style="display: block; margin-bottom: 4px;">ID (optional):</label>
            <input type="text" id="id" name="id" style="width: 100%; padding: 8px; border: 1px solid #ced4da; border-radius: 4px; box-sizing: border-box;" placeholder="auto-generated if empty">
          </div>
          <div style="margin-bottom: 12px;">
            <label for="label" style="display: block; margin-bottom: 4px;">Label (optional):</label>
            <input type="text" id="label" name="label" style="width: 100%; padding: 8px; border: 1px solid #ced4da; border-radius: 4px; box-sizing: border-box;" placeholder="Descriptive name">
          </div>
          <button type="submit" style="background-color: #007bff; color: white; padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer;">Import Wallet</button>
        </form>"#.to_string()
}

fn wallet_table(profiles: &[NeoWalletProfile]) -> String {
    if profiles.is_empty() {
        return html::note("No wallet profiles have been validated in this workspace.");
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
                    r#"<a href="/wallets/{}/delete">Delete</a>"#,
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
