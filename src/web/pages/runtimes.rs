//! Runtimes: the node binaries this workspace has installed, and the catalogues
//! they came from. Inventory only — downloading and installing a runtime writes
//! to the host and stays a deliberate CLI/API action, so the page reports what
//! is there and how well it was verified rather than fetching anything.

use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse, Response},
};

use crate::core::{
    operations::format_bytes,
    runtime::{RuntimeCatalogProfile, RuntimeInstallation},
};

use super::super::{html, WebState};

pub async fn runtimes(State(state): State<WebState>, RawQuery(query): RawQuery) -> Response {
    let body = match render_body(&state) {
        Ok(body) => body,
        Err(error) => html::note(&format!("failed to load runtime inventory: {error}")),
    };
    Html(html::layout(
        "Runtimes",
        "runtimes",
        &html::flash(query.as_deref()),
        &body,
    ))
    .into_response()
}

fn render_body(state: &WebState) -> anyhow::Result<String> {
    let installations = state.repository.list_runtime_installations()?;
    let profiles = state.repository.list_runtime_catalog_profiles()?;
    let verified = installations
        .iter()
        .filter(|installation| installation.signature_verified)
        .count();
    Ok(format!(
        r#"<h1>Runtimes</h1>
{tiles}
<h2>Installed binaries</h2>
{installations}
<h2>Catalog profiles</h2>
{profiles}"#,
        tiles = html::cards(&[
            ("Installed", installations.len().to_string()),
            ("Signature verified", verified.to_string()),
            ("Catalogs", profiles.len().to_string()),
            (
                "On disk",
                format_bytes(
                    installations
                        .iter()
                        .map(|installation| installation.bytes)
                        .sum(),
                ),
            ),
        ]),
        installations = installation_table(&installations),
        profiles = profile_table(&profiles),
    ))
}

fn installation_table(installations: &[RuntimeInstallation]) -> String {
    if installations.is_empty() {
        return html::note("No runtime binaries have been installed into this workspace.");
    }
    let rows = installations
        .iter()
        .map(|installation| {
            html::row(&[
                html::cell(&installation.label),
                html::cell(&installation.node_type.to_string()),
                html::cell(&installation.version),
                html::cell(&installation.platform.to_string()),
                html::cell(&installation.binary_path.display().to_string()),
                html::cell(&short_hash(&installation.sha256)),
                html::raw_cell(&verification_badge(installation.signature_verified)),
                html::cell(&format_bytes(installation.bytes)),
                html::cell(&installation.installed_at_unix.to_string()),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Label",
            "Runtime",
            "Version",
            "Platform",
            "Binary",
            "SHA-256",
            "Signature",
            "Size",
            "Installed (unix)",
        ],
        &rows,
    )
}

fn profile_table(profiles: &[RuntimeCatalogProfile]) -> String {
    if profiles.is_empty() {
        return html::note("No runtime catalog profiles are configured.");
    }
    let rows = profiles
        .iter()
        .map(|profile| {
            html::row(&[
                html::cell(&profile.label),
                html::cell(&profile.id),
                html::cell(&profile.source),
                html::raw_cell(&enabled_badge(profile.enabled)),
                html::cell(profile.last_signature_verified.map_or("never", |verified| {
                    if verified {
                        "yes"
                    } else {
                        "failed"
                    }
                })),
                html::cell(
                    &profile
                        .last_loaded_at_unix
                        .map_or("—".to_string(), |loaded| loaded.to_string()),
                ),
                html::cell(&profile.last_bytes.map_or("—".to_string(), format_bytes)),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Label",
            "Id",
            "Source",
            "Status",
            "Signature",
            "Last loaded (unix)",
            "Last size",
        ],
        &rows,
    )
}

fn verification_badge(verified: bool) -> String {
    badge(verified, "verified", "unverified")
}

fn enabled_badge(enabled: bool) -> String {
    badge(enabled, "enabled", "disabled")
}

fn badge(good: bool, good_label: &str, bad_label: &str) -> String {
    let (class, label) = if good {
        ("badge running", good_label)
    } else {
        ("badge stopped", bad_label)
    };
    format!(r#"<span class="{class}">{label}</span>"#)
}

/// A full digest is noise in a table; the prefix is enough to compare two rows,
/// and the complete value stays on disk in the installation manifest.
fn short_hash(digest: &str) -> String {
    let head = digest.chars().take(12).collect::<String>();
    if digest.chars().count() > head.chars().count() {
        format!("{head}…")
    } else {
        head
    }
}
