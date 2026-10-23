//! The snapshot register form: what the browser posts, and what the page draws
//! back when the workspace refuses it.
//!
//! One struct is both the submission and the redraw source, the way
//! [`crate::web::node_form::NodeDraft`] is. A rejected manifest has nine fields
//! — a label, a source, a download limit, a 64-character digest — and answering
//! it with a redirect meant the operator retyped all of them to fix one.
//!
//! Field names live here rather than in the page, because the page draws the
//! controls and this module says which control a refusal belongs beside. If the
//! two disagreed, an error would sit under the wrong label; keeping the names in
//! one place makes that disagreement impossible to write.

use serde::Deserialize;

use crate::{
    snapshots::NewFastSyncSnapshot,
    types::{Network, NodeType},
};

use super::form_state::{FormDraft, GENERAL};

pub const ID: &str = "id";
pub const LABEL: &str = "label";
pub const NETWORK: &str = "network";
pub const NODE_TYPE: &str = "node_type";
pub const SOURCE_PATH: &str = "source_path";
pub const SOURCE_URL: &str = "source_url";
pub const DOWNLOAD_FILE_NAME: &str = "download_file_name";
pub const DOWNLOAD_MAX_BYTES: &str = "download_max_bytes";
pub const EXPECTED_SHA256: &str = "expected_sha256";

/// Every field the register form carries, in the order the page draws them.
pub const SNAPSHOT_FIELDS: [&str; 9] = [
    ID,
    LABEL,
    NETWORK,
    NODE_TYPE,
    SOURCE_PATH,
    SOURCE_URL,
    DOWNLOAD_FILE_NAME,
    DOWNLOAD_MAX_BYTES,
    EXPECTED_SHA256,
];

/// A blank size limit means "use a generous default" rather than refusing the
/// save: the field guards runaway downloads, and 8 GiB comfortably covers a
/// mainnet fast-sync archive.
const DEFAULT_DOWNLOAD_MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;

/// The register form as posted, every value still the text the browser sent.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct SnapshotDraft {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub network: String,
    #[serde(default)]
    pub node_type: String,
    #[serde(default)]
    pub source_path: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub download_file_name: String,
    #[serde(default)]
    pub download_max_bytes: String,
    #[serde(default)]
    pub expected_sha256: String,
}

impl FormDraft for SnapshotDraft {
    /// The page draws every control from a field constant and asks for its
    /// value by that same constant, so the value beside a label cannot be
    /// another field's. An unknown name reads as blank rather than failing: the
    /// alternative is a panic in a page renderer.
    fn value(&self, field: &str) -> &str {
        match field {
            ID => &self.id,
            LABEL => &self.label,
            NETWORK => &self.network,
            NODE_TYPE => &self.node_type,
            SOURCE_PATH => &self.source_path,
            SOURCE_URL => &self.source_url,
            DOWNLOAD_FILE_NAME => &self.download_file_name,
            DOWNLOAD_MAX_BYTES => &self.download_max_bytes,
            EXPECTED_SHA256 => &self.expected_sha256,
            _ => "",
        }
    }

    fn checked(&self, _field: &str) -> bool {
        false
    }
}

/// What the form could not accept, and where the operator has to type.
#[derive(Debug)]
pub struct SnapshotRefusal {
    /// The `name` of the input the operator has to change.
    pub field: &'static str,
    /// What is wrong with it.
    pub message: String,
}

impl SnapshotRefusal {
    fn at(field: &'static str, message: impl std::fmt::Display) -> Self {
        Self {
            field,
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for SnapshotRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

/// Turn the posted text into the manifest the workspace stores.
///
/// The checks here are the ones whose answer can be pinned to a control: a
/// closed set typed wrong, a number that is not a number, a digest of the wrong
/// shape. Everything else — an URL that is not HTTPS, a file name the downloader
/// would refuse — stays the workspace's own call, and its sentence comes back
/// above the form rather than being guessed onto a field. Guessing is worse than
/// not attributing: an operator sent to a control that was already right has no
/// way to find the one that was not.
pub fn build(draft: &SnapshotDraft) -> Result<NewFastSyncSnapshot, SnapshotRefusal> {
    let network: Network = draft.network.trim().parse().map_err(|_| {
        SnapshotRefusal::at(
            NETWORK,
            format!("{} is not a network", draft.network.trim()),
        )
    })?;
    let node_type: NodeType = draft.node_type.trim().parse().map_err(|_| {
        SnapshotRefusal::at(
            NODE_TYPE,
            format!("{} is not a supported client", draft.node_type.trim()),
        )
    })?;
    let download_max_bytes = parse_max_bytes(&draft.download_max_bytes)
        .map_err(|error| SnapshotRefusal::at(DOWNLOAD_MAX_BYTES, error))?;
    if draft.id.trim().is_empty() {
        return Err(SnapshotRefusal::at(ID, "snapshot id is required"));
    }
    if draft.label.trim().is_empty() {
        return Err(SnapshotRefusal::at(LABEL, "snapshot label is required"));
    }
    let source_path = draft.source_path.trim();
    let source_url = optional(&draft.source_url);
    if source_path.is_empty() && source_url.is_none() {
        // Two ways to name a source and neither was used, so there is no
        // "missing half" to point at: the pair is what failed.
        return Err(SnapshotRefusal::at(
            GENERAL,
            "give the snapshot a source path or an HTTPS URL",
        ));
    }
    // The same digest rule the workspace applies, asked here so a typo in a
    // 64-character string lands on the field that holds it instead of coming
    // back as a sentence about the manifest as a whole.
    crate::snapshots::normalize_sha256(&draft.expected_sha256)
        .map_err(|error| SnapshotRefusal::at(EXPECTED_SHA256, error))?;

    Ok(NewFastSyncSnapshot {
        id: draft.id.trim().to_string(),
        label: draft.label.trim().to_string(),
        network,
        node_type,
        source_path: std::path::PathBuf::from(source_path),
        source_url,
        download_file_name: optional(&draft.download_file_name),
        download_max_bytes,
        expected_sha256: draft.expected_sha256.trim().to_string(),
    })
}

/// Trim a form field to `None` when it is blank, `Some` otherwise.
fn optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// A blank size limit falls back to the default; anything else must be a whole
/// number so a hand-edited post cannot silently save nonsense.
fn parse_max_bytes(raw: &str) -> anyhow::Result<u64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(DEFAULT_DOWNLOAD_MAX_BYTES);
    }
    trimmed
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("download size limit must be a whole number of bytes"))
}

#[cfg(test)]
#[path = "../../tests/unit/web/snapshot_form/tests.rs"]
mod tests;
