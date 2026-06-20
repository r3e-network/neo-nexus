use super::super::*;

pub(in crate::private_network) fn validate_signer_wallet_path(value: &str) -> Result<PathBuf> {
    let path = value.trim();
    if path.is_empty() {
        anyhow::bail!("signer wallet path is empty");
    }
    if path.contains("://") {
        anyhow::bail!("signer wallet path must be a local filesystem path");
    }
    if path.chars().any(char::is_control) {
        anyhow::bail!("signer wallet path contains control characters");
    }
    Ok(PathBuf::from(path))
}
