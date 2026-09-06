use std::{fs::File, io::Read, path::Path};

use anyhow::{bail, Context, Result};
use zeroize::Zeroizing;

use crate::{secret_file::read_secret, wallet::crypto::sha256_hex};

use super::document::WalletDocument;

const MAX_WALLET_BYTES: u64 = 2 * 1024 * 1024;
const MAX_PASSWORD_BYTES: u64 = 4 * 1024;

pub(super) struct ReadWallet {
    pub document: WalletDocument,
    pub sha256: String,
}

pub(super) fn read_wallet(path: &Path) -> Result<ReadWallet> {
    read_wallet_with_pin(path, None)
}

pub(super) fn read_wallet_pinned(path: &Path, expected_sha256: &str) -> Result<ReadWallet> {
    read_wallet_with_pin(path, Some(expected_sha256))
}

fn read_wallet_with_pin(path: &Path, expected_sha256: Option<&str>) -> Result<ReadWallet> {
    reject_symlink(path, "local wallet")?;
    let bytes = read_bounded(path, MAX_WALLET_BYTES, "local wallet")?;
    let sha256 = sha256_hex(&bytes);
    if expected_sha256.is_some_and(|expected| expected != sha256) {
        bail!(
            "local wallet file changed after startup; restart NeoNexus to review and pin the new wallet"
        );
    }
    let document = serde_json::from_slice::<WalletDocument>(&bytes)
        .with_context(|| format!("failed to parse local wallet {}", path.display()))?;
    Ok(ReadWallet { document, sha256 })
}

pub(super) fn read_password(path: &Path) -> Result<Zeroizing<String>> {
    let bytes = read_secret(path, MAX_PASSWORD_BYTES, "local wallet password")?;
    let text = std::str::from_utf8(&bytes).with_context(|| {
        format!(
            "local wallet password file {} must contain valid UTF-8",
            path.display()
        )
    })?;
    let line = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(text);
    if line.is_empty() || line.contains(['\r', '\n']) {
        bail!(
            "local wallet password file {} must contain exactly one non-empty line",
            path.display()
        );
    }
    Ok(Zeroizing::new(line.to_string()))
}

fn read_bounded(path: &Path, limit: u64, label: &str) -> Result<Zeroizing<Vec<u8>>> {
    let file =
        File::open(path).with_context(|| format!("failed to open {label} {}", path.display()))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("failed to inspect {label} {}", path.display()))?;
    if !metadata.is_file() {
        bail!("{label} path {} is not a regular file", path.display());
    }
    if metadata.len() > limit {
        bail!(
            "{label} file {} exceeds the {limit}-byte limit",
            path.display()
        );
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("failed to read {label} {}", path.display()))?;
    if bytes.len() as u64 > limit {
        bail!(
            "{label} file {} exceeds the {limit}-byte limit",
            path.display()
        );
    }
    Ok(bytes)
}

fn reject_symlink(path: &Path, label: &str) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect {label} path {}", path.display()))?;
    if metadata.file_type().is_symlink() {
        bail!(
            "{label} path {} must not be a symbolic link",
            path.display()
        );
    }
    Ok(())
}
