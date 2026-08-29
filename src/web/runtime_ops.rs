//! The controlled path that puts a node binary on this host.
//!
//! Two rules shape this. First, the browser never supplies a download URL: the
//! page sends a profile id and a release id, and the server resolves the URL by
//! re-reading the catalogue. Otherwise the form could be pointed at any HTTPS
//! host and the server would fetch it. Second, nothing is written to the install
//! root until the package has been verified — `RuntimePackageManager::install`
//! checks the checksum, the platform and the signature before it copies, and a
//! failure there leaves the host untouched.
//!
//! The work runs as a [`crate::web::jobs`] job, so a multi-minute download does
//! not hold a request open and a page reload still shows it running.

use crate::{
    core::{
        operations::format_bytes,
        runtime::{RuntimeCatalogProfile, RuntimePackageManager, RuntimePlatform, RuntimeRelease},
    },
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    repository::Repository,
};

use super::WebState;

/// One runtime job at a time, across fetch and install together: two concurrent
/// installs could interleave writes into the same tree.
pub const LANE: &str = "runtime";

/// A release the operator has chosen, resolved from the catalogue.
pub struct Staged {
    pub profile: RuntimeCatalogProfile,
    pub release: RuntimeRelease,
}

impl Staged {
    /// Whether this release can run here at all. Checked before any download so
    /// a wrong-platform package never spends bandwidth or disk.
    pub fn fits_this_host(&self) -> bool {
        self.release.platform_matches(&RuntimePlatform::current())
    }
}

/// Resolve a profile and one of its releases by id, reading the catalogue from
/// the profile's own source.
pub fn stage(state: &WebState, profile_id: &str, release_id: &str) -> anyhow::Result<Staged> {
    let profile = state
        .repository
        .list_runtime_catalog_profiles()?
        .into_iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| anyhow::anyhow!("catalog profile {profile_id} was not found"))?;
    if !profile.enabled {
        anyhow::bail!("catalog profile {} is disabled", profile.label);
    }
    let load =
        RuntimePackageManager::load_release_catalog(&profile.load_request()).map_err(|error| {
            anyhow::anyhow!("catalogue {} could not be read: {error}", profile.source)
        })?;
    let release = load
        .catalog
        .get(release_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("release {release_id} is not in that catalogue"))?;
    let _ = state
        .repository
        .mark_runtime_catalog_profile_loaded(&profile.id, &load);
    Ok(Staged { profile, release })
}

/// Download, verify and install a staged release. Returns the sentence the page
/// shows; every failure leaves the install root untouched.
fn apply(state: &WebState, staged: &Staged) -> anyhow::Result<String> {
    let release = &staged.release;
    if !staged.fits_this_host() {
        anyhow::bail!(
            "{} is built for {package}, this host is {host}",
            release.label,
            package = release.platform,
            host = RuntimePlatform::current(),
        );
    }
    if already_installed(&state.repository, release)? {
        anyhow::bail!(
            "{} {} for {} is already installed",
            release.node_type,
            release.version,
            release.platform
        );
    }

    let download = RuntimePackageManager::download_https(
        &release.download_request(),
        state.workspace_child_dir("runtime-downloads"),
    )
    .map_err(|error| anyhow::anyhow!("download failed: {error}"))?;
    let manifest = release.manifest_for_source(&download.path);
    let verification = RuntimePackageManager::verify(&manifest)
        .map_err(|error| anyhow::anyhow!("verification failed: {error}"))?;
    if !verification.matches {
        anyhow::bail!(
            "checksum mismatch: catalogue says {}, the file is {}",
            verification.expected_sha256,
            verification.sha256
        );
    }
    if verification.signature_verified == Some(false) {
        anyhow::bail!("signature verification failed; nothing was installed");
    }

    let installation =
        RuntimePackageManager::install(&manifest, state.workspace_child_dir("runtimes"))
            .map_err(|error| anyhow::anyhow!("install failed: {error}"))?;
    state
        .repository
        .upsert_runtime_installation(&installation)?;
    record(
        state,
        EventKind::RuntimeInstalled,
        if verification.signature_verified == Some(true) {
            EventSeverity::Info
        } else {
            // Unsigned is allowed by policy but should not pass unnoticed.
            EventSeverity::Warning
        },
        format!(
            "installed {} {} ({} bytes, sha256 {})",
            installation.node_type,
            installation.version,
            format_bytes(installation.bytes),
            installation.sha256.chars().take(12).collect::<String>(),
        ),
    );
    Ok(format!(
        "{} {} installed to {}",
        installation.node_type,
        installation.version,
        installation.binary_path.display()
    ))
}

fn already_installed(repository: &Repository, release: &RuntimeRelease) -> anyhow::Result<bool> {
    Ok(repository
        .list_runtime_installations()?
        .into_iter()
        .any(|existing| {
            existing.node_type == release.node_type
                && existing.version == release.version
                && existing.platform == release.platform
        }))
}

/// The job body for an install request. Errors become the job's visible detail
/// rather than a panic on a worker thread.
pub fn install_job(
    state: WebState,
    profile_id: String,
    release_id: String,
) -> Result<String, String> {
    let staged = stage(&state, &profile_id, &release_id).map_err(|error| error.to_string())?;
    let label = format!(
        "{} {} ({})",
        staged.release.node_type, staged.release.version, staged.release.platform
    );
    apply(&state, &staged)
        .map(|message| format!("{label}: {message}"))
        .map_err(|error| error.to_string())
}

/// A plain-text summary of what a staged release would do, for the review step.
/// Reads the catalogue only; writes nothing.
pub fn review_lines(staged: &Staged) -> Vec<(&'static str, String)> {
    let release = &staged.release;
    vec![
        ("Catalogue", staged.profile.label.clone()),
        ("Source", staged.profile.source.clone()),
        ("Release", release.label.clone()),
        ("Runtime", release.node_type.to_string()),
        ("Version", release.version.clone()),
        ("Package platform", release.platform.to_string()),
        ("This host", RuntimePlatform::current().to_string()),
        ("Size limit", format_bytes(release.max_bytes)),
        (
            "Expected SHA-256",
            release.expected_sha256.chars().take(24).collect(),
        ),
        (
            "Signature",
            match staged.profile.ed25519_public_key.as_deref() {
                Some(_) => "key configured, verified during install",
                None => "no signer key configured",
            }
            .to_string(),
        ),
    ]
}

fn record(state: &WebState, kind: EventKind, severity: EventSeverity, message: String) {
    // The install already happened; failing to journal it must not be reported
    // as if the install itself had failed.
    let _ = state.repository.record_event(NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind,
        severity,
        message,
    });
}

#[cfg(test)]
#[path = "../../tests/unit/web/runtime_ops/tests.rs"]
mod tests;
