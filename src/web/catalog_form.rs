//! The runtime-catalogue register form: what the browser posts, and which
//! control a refusal belongs beside.
//!
//! A catalogue is six fields, two of them optional and one of them a size limit
//! that has to parse. A rejected one used to answer with a redirect, so an
//! operator who mistyped the number at the bottom retyped the identifier, the
//! label, the source and the optional signature pair above it.
//!
//! The names live here rather than in the page because the page draws the
//! controls and this module says which control a refusal belongs beside. Kept
//! apart, the two would drift; kept here, an error cannot be drawn under the
//! wrong label.

use serde::Deserialize;

use crate::runtime::RuntimeCatalogProfile;

use super::form_state::FormDraft;

pub const CATALOG_ID: &str = "id";
pub const CATALOG_LABEL: &str = "label";
pub const CATALOG_SOURCE: &str = "source";
pub const CATALOG_SIGNATURE_SOURCE: &str = "signature_source";
pub const CATALOG_PUBLIC_KEY: &str = "ed25519_public_key";
pub const CATALOG_MAX_BYTES: &str = "max_bytes";

/// Every field the form carries, in the order the page draws them.
pub const CATALOG_FIELDS: [&str; 6] = [
    CATALOG_ID,
    CATALOG_LABEL,
    CATALOG_SOURCE,
    CATALOG_SIGNATURE_SOURCE,
    CATALOG_PUBLIC_KEY,
    CATALOG_MAX_BYTES,
];

/// How large a catalogue document may be. A catalogue is a manifest, not a
/// payload, so this is generous rather than tuned.
pub const DEFAULT_CATALOG_MAX_BYTES: u64 = 4 * 1024 * 1024;

/// The catalogue form as posted, every value still the text the browser sent.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct CatalogDraft {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub signature_source: String,
    #[serde(default)]
    pub ed25519_public_key: String,
    #[serde(default)]
    pub max_bytes: String,
}

impl FormDraft for CatalogDraft {
    fn value(&self, field: &str) -> &str {
        match field {
            CATALOG_ID => &self.id,
            CATALOG_LABEL => &self.label,
            CATALOG_SOURCE => &self.source,
            CATALOG_SIGNATURE_SOURCE => &self.signature_source,
            CATALOG_PUBLIC_KEY => &self.ed25519_public_key,
            CATALOG_MAX_BYTES => &self.max_bytes,
            _ => "",
        }
    }

    fn checked(&self, _field: &str) -> bool {
        false
    }
}

/// What the form could not accept, and where the operator has to type.
#[derive(Debug)]
pub struct CatalogRefusal {
    /// The `name` of the input the operator has to change.
    pub field: &'static str,
    /// What is wrong with it.
    pub message: String,
}

impl CatalogRefusal {
    fn at(field: &'static str, message: impl std::fmt::Display) -> Self {
        Self {
            field,
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for CatalogRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

/// Turn the posted text into the profile the workspace stores.
///
/// Only the questions this form can answer about one control are answered here:
/// the three fields that must not be blank, and the size limit that must be a
/// number. Everything else — whether the source is an HTTPS URL or a readable
/// path, whether the signature source and the key were given together, whether
/// the key decodes to exactly 32 bytes — is one sentence from the domain about
/// the *combination*, and pinning that sentence to one of three controls would
/// be a guess. The guess is worse than the sentence: an operator sent to a
/// control that was already right has no way to find the one that was not.
pub fn build(draft: &CatalogDraft) -> Result<RuntimeCatalogProfile, CatalogRefusal> {
    let max_bytes = match draft.max_bytes.trim() {
        "" => DEFAULT_CATALOG_MAX_BYTES,
        raw => raw.parse::<u64>().map_err(|_| {
            CatalogRefusal::at(
                CATALOG_MAX_BYTES,
                "size limit must be a whole number of bytes",
            )
        })?,
    };
    if draft.id.trim().is_empty() {
        return Err(CatalogRefusal::at(
            CATALOG_ID,
            "runtime catalog profile id is required",
        ));
    }
    if draft.label.trim().is_empty() {
        return Err(CatalogRefusal::at(
            CATALOG_LABEL,
            "runtime catalog profile label is required",
        ));
    }
    if draft.source.trim().is_empty() {
        return Err(CatalogRefusal::at(
            CATALOG_SOURCE,
            "runtime catalog source is required",
        ));
    }
    Ok(RuntimeCatalogProfile {
        id: draft.id.trim().to_string(),
        label: draft.label.trim().to_string(),
        source: draft.source.trim().to_string(),
        signature_source: optional(&draft.signature_source),
        ed25519_public_key: optional(&draft.ed25519_public_key),
        max_bytes,
        enabled: true,
        last_loaded_at_unix: None,
        last_signature_verified: None,
        last_bytes: None,
    })
}

/// Trim a form field to `None` when it is blank, `Some` otherwise.
fn optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
#[path = "../../tests/unit/web/catalog_form/tests.rs"]
mod tests;
