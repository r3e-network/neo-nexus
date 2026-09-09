//! Installing a plugin package from the browser.
//!
//! The operator uploads a signed-off `.zip` and the four facts that describe it
//! — which node, which plugin id, a human label, and the SHA-256 they expect the
//! archive to have. Nothing is trusted from the upload alone: the bytes are
//! streamed to a temporary file under a bounded cap, and the real verification
//! (checksum, `.zip` shape, safe extraction, atomic install) happens inside
//! `PluginPackageManager::install`, exactly as the CLI reaches it.
//!
//! A 2 GiB package must not hold a request open, so the install itself runs as a
//! [`crate::web::jobs`] job and the handler returns at once. The uploaded temp
//! file is removed once the job settles, whether it succeeded or failed.

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use axum::{
    extract::{Multipart, State},
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    catalog::PluginId,
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    plugins::{PluginPackageManager, PluginPackageManifest},
    types::{node_workspace_path, NodeConfig, NodeType},
};

use super::{html, WebState};

/// One plugin install at a time: two concurrent unpacks into the same node's
/// `Plugins` tree could interleave writes.
pub const LANE: &str = "plugin";

/// Where uploaded packages land before they are verified. A workspace child so
/// it sits beside the database like the runtime and snapshot caches do.
const UPLOAD_DIR: &str = "plugin-uploads";

/// The workspace child that holds each node's managed working directory, the
/// same `nodes/<id>` convention the rest of the workbench uses.
const NODES_DIR: &str = "nodes";

/// The cap enforced while the upload streams in, so an oversize package is
/// refused before it is fully written rather than after. Mirrors the core
/// installer's own limit.
const MAX_PACKAGE_BYTES: u64 = crate::plugins::PLUGIN_PACKAGE_MAX_BYTES;

/// Accept an uploaded plugin package and hand the install to a background job.
///
/// The handler only ever writes the upload to a temp file and validates the
/// surrounding facts; every decision that could touch the node's tree is left to
/// `PluginPackageManager::install` inside the job.
pub async fn install_plugin(State(state): State<WebState>, mut multipart: Multipart) -> Response {
    let upload_dir = state.workspace_child_dir(UPLOAD_DIR);
    if let Err(error) = fs::create_dir_all(&upload_dir) {
        return redirect_back("", &format!("failed: could not prepare uploads: {error}"));
    }
    let temp_path = upload_dir.join(format!("plugin-{}.zip", uuid::Uuid::new_v4().simple()));

    let mut node_id = String::new();
    let mut plugin_id_raw = String::new();
    let mut label = String::new();
    let mut expected_sha256 = String::new();
    let mut received_package = false;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                let _ = fs::remove_file(&temp_path);
                return redirect_back(&node_id, &format!("failed: could not read upload: {error}"));
            }
        };
        match field.name() {
            Some("node_id") => {
                node_id = match field.text().await {
                    Ok(value) => value.trim().to_string(),
                    Err(error) => {
                        let _ = fs::remove_file(&temp_path);
                        return redirect_back(&node_id, &format!("failed: {error}"));
                    }
                }
            }
            Some("plugin_id") => {
                plugin_id_raw = match field.text().await {
                    Ok(value) => value.trim().to_string(),
                    Err(error) => {
                        let _ = fs::remove_file(&temp_path);
                        return redirect_back(&node_id, &format!("failed: {error}"));
                    }
                }
            }
            Some("label") => {
                label = match field.text().await {
                    Ok(value) => value.trim().to_string(),
                    Err(error) => {
                        let _ = fs::remove_file(&temp_path);
                        return redirect_back(&node_id, &format!("failed: {error}"));
                    }
                }
            }
            Some("expected_sha256") => {
                expected_sha256 = match field.text().await {
                    Ok(value) => value.trim().to_string(),
                    Err(error) => {
                        let _ = fs::remove_file(&temp_path);
                        return redirect_back(&node_id, &format!("failed: {error}"));
                    }
                }
            }
            Some("package") => {
                received_package = true;
                if let Err(error) = stream_to_file(field, &temp_path).await {
                    let _ = fs::remove_file(&temp_path);
                    return redirect_back(&node_id, &format!("failed: {error}"));
                }
            }
            _ => {}
        }
    }

    let node_id = node_id.trim().to_string();
    let prepared = prepare_install(
        &state,
        &node_id,
        &plugin_id_raw,
        label,
        expected_sha256,
        received_package,
        &temp_path,
    );
    let (node, manifest) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            return redirect_back(&node_id, &format!("failed: {error}"));
        }
    };

    let node_work_dir = match node_workspace_path(state.workspace_child_dir(NODES_DIR), &node.id) {
        Ok(path) => path,
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            return redirect_back(&node_id, &format!("failed: {error}"));
        }
    };

    let description = format!(
        "install plugin {} ({}) on {}",
        manifest.plugin_id, manifest.label, node.name
    );
    let job_state = state.clone();
    let closure_temp = temp_path.clone();
    let message = match state.jobs.submit(LANE, description, move || {
        install_plugin_job(&job_state, &manifest, &node, &node_work_dir, &closure_temp)
    }) {
        Ok(job) => format!("install started: {}", job.description),
        Err(busy) => {
            // The job never ran, so its cleanup never fires: remove the upload here.
            let _ = fs::remove_file(&temp_path);
            format!("not started: {} is already running", busy.description)
        }
    };
    redirect_back(&node_id, &message)
}

/// Stream an uploaded field to `path`, refusing anything past the size cap
/// before the whole body is written.
async fn stream_to_file(
    mut field: axum::extract::multipart::Field<'_>,
    path: &Path,
) -> anyhow::Result<()> {
    let mut file = File::create(path)
        .map_err(|error| anyhow::anyhow!("could not open upload file: {error}"))?;
    let mut written: u64 = 0;
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|error| anyhow::anyhow!("could not read upload: {error}"))?
    {
        written = written.saturating_add(chunk.len() as u64);
        if written > MAX_PACKAGE_BYTES {
            anyhow::bail!("plugin package exceeds the {MAX_PACKAGE_BYTES}-byte limit");
        }
        file.write_all(&chunk)
            .map_err(|error| anyhow::anyhow!("could not write upload: {error}"))?;
    }
    file.flush()
        .map_err(|error| anyhow::anyhow!("could not finish upload: {error}"))?;
    Ok(())
}

/// Validate the surrounding facts and resolve the target node. The manifest is
/// built here so the job body carries an owned, already-checked request.
fn prepare_install(
    state: &WebState,
    node_id: &str,
    plugin_id_raw: &str,
    label: String,
    expected_sha256: String,
    received_package: bool,
    temp_path: &Path,
) -> anyhow::Result<(NodeConfig, PluginPackageManifest)> {
    if node_id.is_empty() {
        anyhow::bail!("a node is required");
    }
    if !received_package {
        anyhow::bail!("a .zip plugin package is required");
    }
    let plugin_id: PluginId = plugin_id_raw
        .parse()
        .map_err(|_| anyhow::anyhow!("{plugin_id_raw} is not a plugin"))?;

    let node = state
        .repository
        .list_nodes()?
        .into_iter()
        .find(|node| node.id == node_id)
        .ok_or_else(|| anyhow::anyhow!("node {node_id} was not found"))?;
    if node.node_type != NodeType::NeoCli {
        anyhow::bail!("plugin packages are supported for neo-cli nodes only");
    }
    if node.status.is_active() || node.pid.is_some() {
        anyhow::bail!("stop and settle {} before installing a plugin", node.name);
    }

    let manifest = PluginPackageManifest {
        plugin_id,
        label,
        source_path: temp_path.to_path_buf(),
        expected_sha256,
    };
    Ok((node, manifest))
}

/// The job body: verify and install the package, persist the installation, then
/// journal it. The uploaded temp file is removed once the work settles, so a
/// failed verification leaves nothing behind either.
fn install_plugin_job(
    state: &WebState,
    manifest: &PluginPackageManifest,
    node: &NodeConfig,
    node_work_dir: &Path,
    temp_path: &Path,
) -> Result<String, String> {
    let outcome = (|| -> anyhow::Result<String> {
        let installation = PluginPackageManager::install(manifest, node, node_work_dir)
            .map_err(|error| anyhow::anyhow!("install failed: {error}"))?;
        state.repository.upsert_plugin_installation(&installation)?;
        let message = format!(
            "installed {} ({}) on {} — {} files",
            installation.plugin_id, manifest.label, node.name, installation.installed_files
        );
        // The install already happened; a journal failure must not be reported
        // as if the install itself had failed.
        let _ = state.repository.record_event(NewRuntimeEvent {
            node_id: Some(node.id.clone()),
            node_name: Some(node.name.clone()),
            kind: EventKind::PluginInstalled,
            severity: EventSeverity::Info,
            message: message.clone(),
        });
        Ok(message)
    })();
    let _ = fs::remove_file(temp_path);
    outcome.map_err(|error| error.to_string())
}

/// Send the browser back to the plugins page for the node it was working on,
/// carrying the outcome as a flash message.
fn redirect_back(node_id: &str, message: &str) -> Response {
    Redirect::to(&format!(
        "/plugins?node={}&flash={}",
        html::urlencoding_lite(node_id),
        html::urlencoding_lite(message)
    ))
    .into_response()
}
