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
{tiles}
{filters}
{table}
{privacy}"#,
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
        filters = html::filter_form("/wallets", &[("used", &params.used), ("q", &params.q)]),
        table = wallet_table(visible),
        privacy = html::note(
            "Only validation metadata is stored: no private keys, passwords, or wallet bytes ever reach this page.",
        ),
    )
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
        ],
        &rows,
    )
}
