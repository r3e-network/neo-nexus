use std::path::Path;

use anyhow::Result;

use super::{profile, validation, NeoWalletProfile, NeoWalletValidationReport};

pub struct NeoWalletValidator;

impl NeoWalletValidator {
    pub fn validate_path(path: impl AsRef<Path>) -> Result<NeoWalletValidationReport> {
        validation::validate_path(path)
    }

    pub fn profile_from_path(
        path: impl AsRef<Path>,
        id: impl Into<String>,
        label: impl Into<String>,
        validated_at_unix: u64,
    ) -> Result<NeoWalletProfile> {
        profile::profile_from_path(path, id, label, validated_at_unix)
    }
}
