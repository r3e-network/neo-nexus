use std::path::Path;

use anyhow::Result;

mod import;
mod primary;
mod validate;

use super::NeoWalletProfile;

pub use validate::validate_neo_wallet_profile;

pub(super) fn profile_from_path(
    path: impl AsRef<Path>,
    id: impl Into<String>,
    label: impl Into<String>,
    validated_at_unix: u64,
) -> Result<NeoWalletProfile> {
    import::profile_from_path(path, id, label, validated_at_unix)
}
