//! Runtimes: the node binaries this workspace has installed, the catalogues
//! they came from, and the controlled path to install a new one.
//!
//! Browsing a catalogue is read-only. Installing is a deliberate action that
//! runs as a background job, and the page shows what would be installed —
//! version, package platform against this host's, size limit, expected digest,
//! signature posture — before the button is offered.

use axum::{
    extract::{Form, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::core::{
    operations::format_bytes,
    runtime::{RuntimeCatalogProfile, RuntimeInstallation, RuntimePackageManager, RuntimePlatform},
};

use super::super::{
    html,
    jobs::JobStatus,
    runtime_ops::{self, LANE},
    time, WebState,
};

#[derive(Default, Deserialize)]
pub struct RuntimeQuery {
    #[serde(default)]
    profile: String,
    #[serde(default)]
    release: String,
}

#[derive(Deserialize)]
pub struct InstallForm {
    #[serde(default)]
    profile: String,
    #[serde(default)]
    release: String,
}

pub async fn runtimes(
    State(state): State<WebState>,
    RawQuery(flash): RawQuery,
    Query(params): Query<RuntimeQuery>,
) -> Response {
    let body = match render_body(&state, &params) {
        Ok(body) => body,
        Err(error) => html::note(&format!("failed to load runtime inventory: {error}")),
    };
    Html(html::layout(
        "Runtimes",
        "runtimes",
        &html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

/// Ask for an install. The work goes to a job thread and this returns at once,
/// so a slow mirror cannot time out the browser.
pub async fn install(State(state): State<WebState>, Form(form): Form<InstallForm>) -> Response {
    let engine = state.clone();
    let profile = form.profile.clone();
    let release = form.release.clone();
    let message = match state.jobs.submit(
        LANE,
        format!(
            "install runtime {}/{}",
            or_unknown(&profile),
            or_unknown(&release)
        ),
        move || runtime_ops::install_job(engine, profile, release),
    ) {
        Ok(job) => format!("install started: {}", job.description),
        Err(busy) => format!("not started: {} is already running", busy.description),
    };
    Redirect::to(&format!(
        "/runtimes?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}

fn or_unknown(value: &str) -> &str {
    if value.trim().is_empty() {
        "?"
    } else {
        value
    }
}

fn render_body(state: &WebState, params: &RuntimeQuery) -> anyhow::Result<String> {
    let installations = state.repository.list_runtime_installations()?;
    let profiles = state.repository.list_runtime_catalog_profiles()?;
    let verified = installations
        .iter()
        .filter(|installation| installation.signature_verified)
        .count();
    Ok(format!(
        r#"<h1>Runtimes</h1>
{tiles}
{jobs}
<h2>Installed binaries</h2>
{installations}
<h2>Catalog profiles</h2>
{profiles}
{staged}"#,
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
        jobs = job_panel(state),
        installations = installation_table(&installations),
        profiles = profile_table(&profiles),
        staged = catalogue_section(state, params)?,
    ))
}

/// What is running now, and what the last few attempts produced.
fn job_panel(state: &WebState) -> String {
    let recent = state.jobs.recent();
    if recent.is_empty() {
        return String::new();
    }
    let running = recent
        .iter()
        .filter(|job| job.status == JobStatus::Running)
        .count();
    let rows = recent
        .iter()
        .take(5)
        .map(|job| {
            html::row(&[
                html::raw_cell(&status_badge(&job.status)),
                html::cell(&job.description),
                html::cell(&job.detail),
                html::raw_cell(&time::time_cell(Some(job.started_at_unix))),
            ])
        })
        .collect::<Vec<_>>();
    let banner = if running > 0 {
        html::notice(
            "warn",
            &format!("{running} job(s) running; this page refreshes as each finishes."),
        )
    } else {
        String::new()
    };
    let table = html::table(&["State", "Work", "Result", "Started"], &rows);
    // The poll marker wraps only the running case: an idle page must not
    // reload itself.
    let body = if running > 0 {
        format!(r#"<div data-job-poll="4000" aria-live="polite">{table}</div>"#)
    } else {
        table
    };
    format!("<h2>Background work</h2>\n{banner}\n{body}")
}

fn status_badge(status: &JobStatus) -> String {
    let class = match status {
        JobStatus::Running => "badge starting",
        JobStatus::Succeeded => "badge running",
        JobStatus::Failed => "badge error",
    };
    format!(
        r#"<span class="{class}">{}</span>"#,
        html::escape(status.label())
    )
}

/// The catalogue for the chosen profile, and the review step for a release.
fn catalogue_section(state: &WebState, params: &RuntimeQuery) -> anyhow::Result<String> {
    let profile_id = params.profile.trim();
    if profile_id.is_empty() {
        return browse_prompt(state);
    }
    let Some(profile) = state
        .repository
        .list_runtime_catalog_profiles()?
        .into_iter()
        .find(|profile| profile.id == profile_id)
    else {
        return Ok(html::note("That catalog profile no longer exists."));
    };
    let header = format!("<h2>Catalogue: {}</h2>", html::escape(&profile.label));
    let Ok(load) = RuntimePackageManager::load_release_catalog(&profile.load_request()) else {
        return Ok(format!(
            "{header}\n{}",
            html::notice(
                "danger",
                &format!(
                    "The catalogue at {} could not be read. Nothing was downloaded.",
                    profile.source
                )
            ),
        ));
    };
    let platform = RuntimePlatform::current();
    let compatible = load.catalog.compatible_releases(&platform);
    if compatible.is_empty() {
        return Ok(format!(
            "{header}\n{}",
            html::note(&format!(
                "No release in this catalogue is built for {platform}."
            ))
        ));
    }
    let rows = compatible
        .iter()
        .map(|release| {
            let href = format!(
                "/runtimes?profile={}&release={}",
                html::urlencoding_lite(&profile.id),
                html::urlencoding_lite(&release.id)
            );
            html::row(&[
                html::raw_cell(&format!(
                    r#"<a href="{href}">{}</a>"#,
                    html::escape(&release.label)
                )),
                html::cell(&release.node_type.to_string()),
                html::cell(&release.version),
                html::cell(&release.platform.to_string()),
                html::cell(&format_bytes(release.max_bytes)),
            ])
        })
        .collect::<Vec<_>>();
    let list = html::table(
        &["Release", "Runtime", "Version", "Platform", "Size limit"],
        &rows,
    );
    let review = if params.release.trim().is_empty() {
        String::new()
    } else {
        match runtime_ops::stage(state, profile_id, params.release.trim()) {
            Ok(staged) => review_panel(&staged, &profile.id),
            Err(error) => html::notice("danger", &error.to_string()),
        }
    };
    Ok(format!("{header}\n{list}\n{review}"))
}

/// With no profile chosen, offer the enabled ones. Browsing is a deliberate
/// click because it reaches out to the profile's source.
fn browse_prompt(state: &WebState) -> anyhow::Result<String> {
    let links = state
        .repository
        .list_runtime_catalog_profiles()?
        .into_iter()
        .filter(|profile| profile.enabled)
        .map(|profile| {
            format!(
                r#"<a class="btn" href="/runtimes?profile={}">Browse {}</a>"#,
                html::urlencoding_lite(&profile.id),
                html::escape(&profile.label)
            )
        })
        .collect::<Vec<_>>();
    if links.is_empty() {
        return Ok(String::new());
    }
    Ok(format!(
        "<h2>Catalogue</h2>\n{}\n<div class=\"actions\">{}</div>",
        html::note("Reading a catalogue fetches it from its configured source. Nothing is written to this host."),
        links.join("")
    ))
}

/// The review step: what would be installed, and the only place the button is.
fn review_panel(staged: &runtime_ops::Staged, profile_id: &str) -> String {
    let rows = runtime_ops::review_lines(staged)
        .iter()
        .map(|(label, value)| html::row(&[html::cell(label), html::cell(value)]))
        .collect::<Vec<_>>();
    let facts = html::table(&["Setting", "Value"], &rows);
    if !staged.fits_this_host() {
        let notice = html::notice(
            "danger",
            "This release is not built for this host, so it cannot be installed from here.",
        );
        return format!("<h2>Review</h2>\n{facts}\n{notice}");
    }
    let form = html::control_form(
        "/runtimes/install",
        &[("profile", profile_id), ("release", &staged.release.id)],
        &format!(
            "Install {} {}",
            staged.release.node_type, staged.release.version
        ),
    );
    let note = html::note(
        "The package is downloaded, its digest and signature checked, and only then copied into the workspace. A verification failure writes nothing.",
    );
    format!("<h2>Review</h2>\n{facts}\n<div class=\"actions\">{form}</div>\n{note}")
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
                html::raw_cell(&time::time_cell(Some(installation.installed_at_unix))),
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
            "Installed",
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
                html::raw_cell(&time::time_cell(profile.last_loaded_at_unix)),
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
            "Last loaded",
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
