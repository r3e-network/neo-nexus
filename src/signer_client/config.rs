//! Fail-closed endpoint and credential configuration.

use std::{env, fmt, fs, path::Path};

use anyhow::{bail, Context, Result};
use url::{Host, Url};
use zeroize::Zeroizing;

use super::{BearerCredential, SignerCredential, WorkloadCredential};

const URL_VARIABLE: &str = "NEONEXUS_SIGNER_URL";
const MAX_SECRET_FILE_BYTES: u64 = 16 * 1024;

#[derive(Clone)]
pub struct SignerClientConfig {
    pub endpoint: SignerEndpoint,
    pub credential: SignerCredential,
}

impl fmt::Debug for SignerClientConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerClientConfig")
            .field("endpoint", &self.endpoint)
            .field("credential", &self.credential)
            .finish()
    }
}

impl SignerClientConfig {
    /// Read one named credential profile plus the shared signer URL.
    ///
    /// For prefix `NEONEXUS_SIGNER_ADMIN`, bearer mode reads
    /// `_TOKEN_FILE`; workload mode reads `_CALLER_ID`,
    /// `_WORKLOAD_KEY_FILE`, and optional `_WORKLOAD_SUBJECT`.
    pub fn from_env(prefix: &str) -> Result<Option<Self>> {
        validate_prefix(prefix)?;
        let token_name = format!("{prefix}_TOKEN_FILE");
        let caller_name = format!("{prefix}_CALLER_ID");
        let key_name = format!("{prefix}_WORKLOAD_KEY_FILE");
        let subject_name = format!("{prefix}_WORKLOAD_SUBJECT");

        let raw_url = env_text(URL_VARIABLE);
        let token_file = env_text(&token_name);
        let caller_id = env_text(&caller_name);
        let key_file = env_text(&key_name);
        let subject = env_text(&subject_name);
        let any_credential =
            token_file.is_some() || caller_id.is_some() || key_file.is_some() || subject.is_some();

        if raw_url.is_none() && !any_credential {
            return Ok(None);
        }
        let endpoint = SignerEndpoint::parse(
            raw_url
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("{URL_VARIABLE} is required"))?,
        )?;

        let bearer_selected = token_file.is_some();
        let workload_selected = caller_id.is_some() || key_file.is_some() || subject.is_some();
        if bearer_selected == workload_selected {
            bail!("{prefix} must configure exactly one of bearer token file or workload identity");
        }

        let credential = match token_file {
            Some(path) => {
                let token = read_secret_text(Path::new(&path), "signer bearer token")?;
                SignerCredential::Bearer(BearerCredential::new(token.as_str())?)
            }
            None => {
                let caller_id = caller_id
                    .ok_or_else(|| anyhow::anyhow!("{caller_name} is required in workload mode"))?;
                let key_file = key_file
                    .ok_or_else(|| anyhow::anyhow!("{key_name} is required in workload mode"))?;
                let seed = read_secret_text(Path::new(&key_file), "signer workload key")?;
                SignerCredential::Workload(Box::new(WorkloadCredential::from_seed_hex(
                    caller_id, subject, &seed,
                )?))
            }
        };
        Ok(Some(Self {
            endpoint,
            credential,
        }))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SignerEndpoint {
    url: Url,
}

impl SignerEndpoint {
    pub fn parse(raw: &str) -> Result<Self> {
        let mut url = Url::parse(raw.trim()).context("signer URL is invalid")?;
        if !matches!(url.scheme(), "http" | "https") {
            bail!("signer URL scheme must be http or https");
        }
        if !url.username().is_empty() || url.password().is_some() {
            bail!("signer URL must not contain credentials");
        }
        if url.query().is_some() || url.fragment().is_some() {
            bail!("signer URL must not contain a query or fragment");
        }
        if !matches!(url.path(), "" | "/") {
            bail!("signer URL must not contain a path");
        }
        if url.scheme() == "http" && !is_loopback(&url) {
            bail!("cleartext signer URL is allowed only on a loopback IP");
        }
        if url.host().is_none() {
            bail!("signer URL must name a host");
        }
        url.set_path("");
        Ok(Self { url })
    }

    pub fn as_str(&self) -> &str {
        self.url.as_str().trim_end_matches('/')
    }

    pub(crate) fn request_url(&self, route: &str) -> Result<Url> {
        if !route.starts_with('/') {
            bail!("signer route must start with /");
        }
        self.url
            .join(route)
            .context("signer route could not be joined to endpoint")
    }
}

impl fmt::Debug for SignerEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("SignerEndpoint")
            .field(&self.as_str())
            .finish()
    }
}

impl fmt::Display for SignerEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn is_loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        _ => false,
    }
}

fn env_text(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn validate_prefix(prefix: &str) -> Result<()> {
    if prefix.is_empty()
        || prefix
            .bytes()
            .any(|byte| !(byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'))
    {
        bail!("signer credential prefix must use uppercase ASCII, digits, and underscores");
    }
    Ok(())
}

fn read_secret_text(path: &Path, label: &str) -> Result<Zeroizing<String>> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("{label} file could not be inspected: {}", path.display()))?;
    if !metadata.is_file() {
        bail!("{label} path is not a file: {}", path.display());
    }
    if metadata.len() > MAX_SECRET_FILE_BYTES {
        bail!("{label} file is too large");
    }
    let raw = fs::read(path)
        .with_context(|| format!("{label} file could not be read: {}", path.display()))?;
    let text = String::from_utf8(raw).with_context(|| format!("{label} file is not UTF-8"))?;
    Ok(Zeroizing::new(text.trim().to_string()))
}
